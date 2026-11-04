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
pub type Handler<S> = for<'a> fn(&'a mut Session<S>, Call<'a>) -> BoxFut<'a, ()>;

pub struct Call<'a> {
    pub format: Format,
    /// No answer is serialized for a notification.
    pub id: Option<&'a Id>,
    pub params: Params<'a>,
    pub body: &'a mut Vec<u8>,
}

impl Call<'_> {
    pub fn fail(&mut self, code: i32, message: &str) {
        if let Some(id) = self.id {
            envelope::failure_into::<()>(self.format, id, code, message, None, self.body);
        }
    }

    pub fn fail_aux(&mut self, aux: &Aux) {
        if let Some(id) = self.id {
            envelope::failure_into(
                self.format,
                id,
                aux_code(aux),
                &aux.to_string(),
                Some(aux),
                self.body,
            );
        }
    }

    /// Encodes an endpoint's outcome: `result`, a domain error or an auxiliary one.
    pub fn finish<O, E>(&mut self, response: Response<O, E>)
    where
        O: Serialize,
        E: Serialize + fmt::Display,
    {
        let Some(id) = self.id else {
            return;
        };
        match response {
            Ok(ok) => envelope::success_into(self.format, id, &ok, self.body),
            Err(Generic::Spec(err)) => envelope::failure_into(
                self.format,
                id,
                code::DOMAIN,
                &err.to_string(),
                Some(&err),
                self.body,
            ),
            Err(Generic::Aux(aux)) => self.fail_aux(&aux),
        }
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
    mut call: Call<'a>,
) -> BoxFut<'a, ()> {
    Box::pin(async move {
        let args = match call.params.decode::<E>() {
            Ok(args) => args,
            Err(message) => return call.fail(code::INVALID_PARAMS, &message),
        };

        call.finish(E::call(session, args).await)
    })
}

/// Streaming endpoints have no serde form.
macro_rules! endpoints {
    ($(
        $accessor:ident {
            trait: $Trait:path,
            client: $Client:path,
            methods: {$(
                $method:ident {
                    flags: [$($flag:ident),*],
                    lifetimes: [$($lt:lifetime),*],
                    args: $Args:ty,
                    ok: $Ok:ty,
                    err: $Err:ty,
                    output: $Output:ty,
                    docs: [$($doc:literal),*],
                    $($method_extra:tt)*
                }
            )*},
            $($extra:tt)*
        }
    )*) => {
        $($(
            eva::if_flag!(stream in [$($flag),*] {} else {
                impl<S: IsSession> Endpoint<S> for $Args {
                    const METHOD: &'static str = concat!(stringify!($accessor), ".", stringify!($method));

                    type Ok = $Ok;
                    type Err = $Err;

                    fn call(
                        session: &mut Session<S>,
                        args: Self,
                    ) -> impl Future<Output = Response<Self::Ok, Self::Err>> + Send {
                        use eva::handlers::CallStep as _;
                        use $Trait as _;

                        async move { session.$accessor().inner_mut().$method().call(args).await }
                    }
                }
            });
        )*)*

        /// Every serde endpoint of the service, keyed by `<accessor>.<method>`.
        pub fn table<S: IsSession>() -> HashMap<&'static str, Handler<S>> {
            let mut table = HashMap::new();
            $($(
                eva::if_flag!(stream in [$($flag),*] {} else {
                    table.insert(
                        <$Args as Endpoint<S>>::METHOD,
                        handle::<S, $Args> as Handler<S>,
                    );
                });
            )*)*
            table
        }
    };
}

viendesu_core::for_each_endpoint!(endpoints);

#[cfg(test)]
mod tests {
    use super::*;

    struct MustNotEncode;

    impl Serialize for MustNotEncode {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            panic!("notification result must not be serialized");
        }
    }

    impl fmt::Display for MustNotEncode {
        fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
            panic!("notification error must not be formatted");
        }
    }

    #[test]
    fn notifications_do_not_encode_results_or_errors() {
        for format in [Format::Json, Format::Msgpack] {
            let mut body = Vec::new();
            let mut call = Call {
                format,
                id: None,
                params: Params::Absent,
                body: &mut body,
            };
            call.finish::<_, MustNotEncode>(Ok(MustNotEncode));
            call.finish::<MustNotEncode, _>(Err(Generic::Spec(MustNotEncode)));
            call.finish::<MustNotEncode, MustNotEncode>(Err(Generic::Aux(Aux::Unauthenticated)));
            call.fail(code::INVALID_PARAMS, "invalid params");
            assert_eq!(body.capacity(), 0);
        }
    }
}
