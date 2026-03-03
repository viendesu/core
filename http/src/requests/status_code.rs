use http::status::StatusCode;

use viendesu_core::errors;

pub trait HasStatusCode {
    fn status_code(&self) -> StatusCode;
}

impl<O: HasStatusCode, E: HasStatusCode> HasStatusCode for Result<O, E> {
    fn status_code(&self) -> StatusCode {
        match self {
            Ok(o) => o.status_code(),
            Err(e) => e.status_code(),
        }
    }
}

impl<S: HasStatusCode> HasStatusCode for errors::Generic<S> {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Aux(aux) => aux.status_code(),
            Self::Spec(spec) => spec.status_code(),
        }
    }
}

impl HasStatusCode for errors::Aux {
    fn status_code(&self) -> StatusCode {
        use StatusCode as C;
        use errors::Aux::*;

        match self {
            Unauthenticated => C::UNAUTHORIZED,
            InvalidRole(e) => e.status_code(),
            InvalidSession(e) => e.status_code(),
            Captcha(..) | Deserialization(..) => C::BAD_REQUEST,
            Db(..) | InternalError(..) | ObjectStore(..) | Mail(..) => C::INTERNAL_SERVER_ERROR,
        }
    }
}

macro_rules! map {
    ($Ty:ty => [$($Item:ident),* $(,)?]) => {
        const _: () = {
            use $crate::requests::status_code::HasStatusCode;
            use ::http::status::StatusCode;

            impl HasStatusCode for $Ty {
                fn status_code(&self) -> StatusCode {
                    match *self {$(
                        Self::$Item(ref e) => e.status_code(),
                    )*}
                }
            }
        };
    };
}

pub(crate) use map;

macro_rules! direct {
    ($($ty:ty => $code:ident),* $(,)?) => {$(
        const _: () = {
            use $crate::requests::status_code::HasStatusCode;
            use ::http::status::StatusCode;

            impl HasStatusCode for $ty {
                fn status_code(&self) -> StatusCode {
                    StatusCode::$code
                }
            }
        };
    )*};
}

pub(crate) use direct;

direct! {
    errors::auth::InvalidRole => FORBIDDEN,
    errors::auth::InvalidSession => BAD_REQUEST,
}
