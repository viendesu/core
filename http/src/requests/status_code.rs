use http::status::StatusCode;
use schemars::{Schema, SchemaGenerator};

use viendesu_protocol::errors;

pub trait HasStatusCode {
    fn status_code(&self) -> StatusCode;
}

/// Types every value of which maps to the same status code.
pub trait ConstStatusCode {
    const STATUS: StatusCode;
}

/// Wire-level description of one variant of a specific error enum.
pub struct ErrorVariant {
    /// Serialized (snake_case) variant name.
    pub name: String,
    pub status: StatusCode,
    /// Schema of the variant payload.
    pub schema: Schema,
}

/// Static per-variant breakdown of a specific error enum.
pub trait ErrorVariants {
    fn error_variants(generator: &mut SchemaGenerator) -> Vec<ErrorVariant>;
}

pub fn snake_case(ident: &str) -> String {
    let mut out = String::with_capacity(ident.len() + 4);
    for (i, c) in ident.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
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
            use $crate::requests::status_code::{ErrorVariant, ErrorVariants, HasStatusCode};
            use ::http::status::StatusCode;

            impl HasStatusCode for $Ty {
                fn status_code(&self) -> StatusCode {
                    match *self {$(
                        Self::$Item(ref e) => e.status_code(),
                    )*}
                }
            }

            impl ErrorVariants for $Ty {
                #[allow(unused)]
                fn error_variants(
                    generator: &mut ::schemars::SchemaGenerator,
                ) -> Vec<ErrorVariant> {
                    // Recovers the variant's payload type from its constructor,
                    // giving access to the payload's const status and schema.
                    fn probe<T>(
                        _: fn(T) -> $Ty,
                        generator: &mut ::schemars::SchemaGenerator,
                    ) -> (StatusCode, ::schemars::Schema)
                    where
                        T: $crate::requests::status_code::ConstStatusCode
                            + ::schemars::JsonSchema,
                    {
                        (T::STATUS, generator.subschema_for::<T>())
                    }

                    ::std::vec![$({
                        let (status, schema) = probe(<$Ty>::$Item, generator);
                        ErrorVariant {
                            name: $crate::requests::status_code::snake_case(
                                ::core::stringify!($Item),
                            ),
                            status,
                            schema,
                        }
                    }),*]
                }
            }
        };
    };
}

pub(crate) use map;

macro_rules! direct {
    ($($ty:ty => $code:ident),* $(,)?) => {$(
        const _: () = {
            use $crate::requests::status_code::{ConstStatusCode, HasStatusCode};
            use ::http::status::StatusCode;

            impl ConstStatusCode for $ty {
                const STATUS: StatusCode = StatusCode::$code;
            }

            impl HasStatusCode for $ty {
                fn status_code(&self) -> StatusCode {
                    Self::STATUS
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
