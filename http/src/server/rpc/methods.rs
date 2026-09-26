//! Method table generated from [`viendesu_core::for_each_endpoint`].

use std::{collections::HashMap, fmt, future::Future, pin::Pin};

use serde::{Serialize, de::DeserializeOwned};

use viendesu_core::service::{IsSession, Session};
use viendesu_protocol::{
    errors::{Aux, Generic},
    requests::Response,
};

use crate::format::Format;

use super::{
    code,
    envelope::{self, Id, Params},
};

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Decodes the params, runs the endpoint and encodes the response body.
pub type Handler<S> = for<'a> fn(&'a mut Session<S>, Call<'a>) -> BoxFut<'a, Vec<u8>>;

pub struct Call<'a> {
    pub format: Format,
    pub id: &'a Id,
    pub params: Params<'a>,
}

impl Call<'_> {
    pub fn fail(&self, code: i32, message: &str) -> Vec<u8> {
        envelope::failure_plain(self.format, self.id, code, message)
    }

    pub fn fail_aux(&self, aux: &Aux) -> Vec<u8> {
        let message = aux.to_string();
        envelope::failure(self.format, self.id, aux_code(aux), &message, Some(aux))
    }
}

pub const fn aux_code(aux: &Aux) -> i32 {
    match aux {
        Aux::Unauthenticated => code::UNAUTHENTICATED,
        Aux::InvalidRole(_) => code::FORBIDDEN,
        Aux::InvalidSession(_) => code::INVALID_SESSION,
        Aux::Captcha(_) => code::CAPTCHA,
        Aux::Deserialization(_) => code::INVALID_PARAMS,
        Aux::Db(_) | Aux::InternalError(_) | Aux::ObjectStore(_) | Aux::Mail(_) => {
            code::INTERNAL_ERROR
        }
    }
}

/// A service endpoint callable over RPC, implemented for its `Args`.
trait Endpoint<S: IsSession>: DeserializeOwned + Send + 'static {
    const METHOD: &'static str;

    type Ok: Serialize + Send;
    type Err: Serialize + fmt::Display + Send;

    fn call(
        session: &mut Session<S>,
        args: Self,
    ) -> impl Future<Output = Response<Self::Ok, Self::Err>> + Send;
}

fn handle<'a, S: IsSession, E: Endpoint<S>>(
    session: &'a mut Session<S>,
    call: Call<'a>,
) -> BoxFut<'a, Vec<u8>> {
    Box::pin(async move {
        let args = match call.params.decode::<E>() {
            Ok(args) => args,
            Err(message) => return call.fail(code::INVALID_PARAMS, &message),
        };

        match E::call(session, args).await {
            Ok(ok) => envelope::success(call.format, call.id, &ok),
            Err(Generic::Spec(err)) => envelope::failure(
                call.format,
                call.id,
                code::DOMAIN,
                &err.to_string(),
                Some(&err),
            ),
            Err(Generic::Aux(aux)) => call.fail_aux(&aux),
        }
    })
}

macro_rules! endpoint {
    ([stream] $($rest:tt)*) => {};
    ([] $module:ident $accessor:ident $Trait:ident $method:ident []) => {
        endpoint!([] $module $accessor $Trait $method [$method]);
    };
    ([] $module:ident $accessor:ident $Trait:ident $method:ident [$endpoint:ident]) => {
        impl<S: IsSession> Endpoint<S> for viendesu_protocol::requests::$module::$endpoint::Args {
            const METHOD: &'static str = concat!(stringify!($module), ".", stringify!($endpoint));

            type Ok = viendesu_protocol::requests::$module::$endpoint::Ok;
            type Err = viendesu_protocol::requests::$module::$endpoint::Err;

            fn call(
                session: &mut Session<S>,
                args: Self,
            ) -> impl Future<Output = Response<Self::Ok, Self::Err>> + Send {
                use viendesu_core::service::{CallStep as _, $module::$Trait as _};

                async move { session.$accessor().$method().call(args).await }
            }
        }
    };
}

macro_rules! register {
    ($table:ident [stream] $($rest:tt)*) => {};
    ($table:ident [] $module:ident $method:ident []) => {
        register!($table [] $module $method [$method]);
    };
    ($table:ident [] $module:ident $method:ident [$endpoint:ident]) => {{
        type Args = viendesu_protocol::requests::$module::$endpoint::Args;

        let displaced = $table.insert(
            <Args as Endpoint<S>>::METHOD,
            handle::<S, Args> as Handler<S>,
        );
        assert!(displaced.is_none(), "duplicate RPC method {}", <Args as Endpoint<S>>::METHOD);
    }};
}

macro_rules! endpoints {
    ($(
        $module:ident {$(
            $accessor:ident : $Trait:ident {
                $( $(#[$attr:ident])* $method:ident $(=> $endpoint:ident)? ),* $(,)?
            }
        )*}
    )*) => {
        $($($(
            endpoint!([$($attr)*] $module $accessor $Trait $method [$($endpoint)?]);
        )*)*)*

        /// Every serde endpoint of the service, keyed by `<module>.<endpoint>`.
        pub fn table<S: IsSession>() -> HashMap<&'static str, Handler<S>> {
            let mut table = HashMap::new();
            $($($(
                register!(table [$($attr)*] $module $method [$($endpoint)?]);
            )*)*)*
            table
        }
    };
}

viendesu_core::for_each_endpoint!(endpoints);
