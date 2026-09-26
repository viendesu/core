//! A service whose every endpoint answers from a per-method script.

use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Mutex, MutexGuard},
};

use futures::StreamExt as _;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use viendesu_core::service::{
    AuxFut, CallStep, RespFut, Session, SessionMaker, authz::Authentication,
};
use viendesu_protocol::{
    errors::{self, Aux, Generic},
    requests::{self, Response},
    types::session,
    uploads::Chunk,
};

#[derive(Clone, Default)]
pub struct Mock(Arc<Mutex<Script>>);

#[derive(Default)]
struct Script {
    replies: HashMap<&'static str, Value>,
    calls: Vec<(&'static str, Value)>,
    tokens: Vec<session::Token>,
    reject_tokens: bool,
}

impl Mock {
    fn script(&self) -> MutexGuard<'_, Script> {
        self.0.lock().unwrap()
    }

    /// Scripts `method` to answer `{"ok": v}`, `{"err": v}` or `{"aux": v}`.
    pub fn reply(&self, method: &'static str, reply: Value) {
        self.script().replies.insert(method, reply);
    }

    /// Calls made so far, with their args in JSON form.
    pub fn calls(&self) -> Vec<(&'static str, Value)> {
        self.script().calls.clone()
    }

    pub fn tokens(&self) -> Vec<session::Token> {
        self.script().tokens.clone()
    }

    pub fn reject_tokens(&self) {
        self.script().reject_tokens = true;
    }

    fn record(&self, method: &'static str, args: Value) -> Option<Value> {
        let mut script = self.script();
        script.calls.push((method, args));
        script.replies.get(method).cloned()
    }
}

fn canned<O: DeserializeOwned, E: DeserializeOwned>(
    method: &str,
    reply: Option<Value>,
) -> Response<O, E> {
    let Some(reply) = reply else {
        return Err(Generic::Aux(Aux::InternalError(format!(
            "{method} is not mocked"
        ))));
    };
    let (kind, value) = reply
        .as_object()
        .and_then(|reply| reply.iter().next())
        .expect("reply is {\"ok\" | \"err\" | \"aux\": value}");
    // Through text: eva's `Date` only deserializes from borrowed strings.
    let value = value.to_string();

    match kind.as_str() {
        "ok" => Ok(serde_json::from_str(&value).unwrap()),
        "err" => Err(Generic::Spec(serde_json::from_str(&value).unwrap())),
        "aux" => Err(Generic::Aux(serde_json::from_str(&value).unwrap())),
        other => panic!("unknown reply kind {other:?}"),
    }
}

pub struct Step<A, O, E> {
    mock: Mock,
    method: &'static str,
    _types: PhantomData<fn(A) -> (O, E)>,
}

impl<A, O, E> CallStep<A> for Step<A, O, E>
where
    A: Serialize + Send,
    O: DeserializeOwned + Send,
    E: DeserializeOwned + Send,
{
    type Ok = O;
    type Err = E;

    fn call(&mut self, args: A) -> impl RespFut<O, E> {
        let reply = self
            .mock
            .record(self.method, serde_json::to_value(&args).unwrap());
        let method = self.method;
        async move { canned(method, reply) }
    }
}

/// `uploads.finish`: drains the stream and records `{"id", "bytes", "aborted"}`.
pub struct Finish {
    mock: Mock,
}

impl CallStep<requests::uploads::finish::Args> for Finish {
    type Ok = requests::uploads::finish::Ok;
    type Err = requests::uploads::finish::Err;

    fn call(
        &mut self,
        requests::uploads::finish::Args { id, mut stream }: requests::uploads::finish::Args,
    ) -> impl RespFut<Self::Ok, Self::Err> {
        let mock = self.mock.clone();
        async move {
            let mut bytes = 0;
            let mut aborted = None;
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Chunk::Data(data) => bytes += data.len(),
                    Chunk::Aborted(reason) => aborted = Some(reason.to_string()),
                }
            }

            let reply = mock.record(
                "uploads.finish",
                serde_json::json!({ "id": id, "bytes": bytes, "aborted": aborted }),
            );
            canned("uploads.finish", reply)
        }
    }
}

macro_rules! mock_method {
    ($attrs:tt $module:ident $method:ident []) => {
        mock_method!($attrs $module $method [$method]);
    };
    ([stream] $module:ident $method:ident [$endpoint:ident]) => {
        fn $method(
            &mut self,
        ) -> impl CallStep<
            requests::$module::$endpoint::Args,
            Ok = requests::$module::$endpoint::Ok,
            Err = requests::$module::$endpoint::Err,
        > {
            Finish { mock: self.clone() }
        }
    };
    ([] $module:ident $method:ident [$endpoint:ident]) => {
        fn $method(
            &mut self,
        ) -> impl CallStep<
            requests::$module::$endpoint::Args,
            Ok = requests::$module::$endpoint::Ok,
            Err = requests::$module::$endpoint::Err,
        > {
            Step {
                mock: self.clone(),
                method: concat!(stringify!($module), ".", stringify!($endpoint)),
                _types: PhantomData,
            }
        }
    };
}

macro_rules! mock_domains {
    ($(
        $module:ident {$(
            $accessor:ident : $Trait:ident {
                $( $(#[$attr:ident])* $method:ident $(=> $endpoint:ident)? ),* $(,)?
            }
        )*}
    )*) => {
        $($(
            impl viendesu_core::service::$module::$Trait for Mock {
                $( mock_method!([$($attr)*] $module $method [$($endpoint)?]); )*
            }
        )*)*
    };
}

viendesu_core::for_each_endpoint!(mock_domains);

impl Authentication for Mock {
    fn authenticate(&mut self, token: session::Token) -> impl AuxFut<()> {
        let rejected = {
            let mut script = self.script();
            script.tokens.push(token);
            script.reject_tokens
        };

        async move {
            if rejected {
                Err(Aux::InvalidSession(errors::auth::InvalidSession { token }))
            } else {
                Ok(())
            }
        }
    }

    fn clear(&mut self) {}
}

#[derive(Clone, Default)]
pub struct Service(pub Mock);

impl SessionMaker for Service {
    type Session = Mock;

    fn make_session(&self) -> impl AuxFut<Session<Mock>> {
        let mock = self.0.clone();
        async move { Ok(Session::new(mock)) }
    }
}

pub struct Types;

impl viendesu_http::server::Types for Types {
    type Service = Service;
}
