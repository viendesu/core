use eva::{data, str::CompactString};
use std::fmt;

pub mod ext;

pub mod boards;
pub mod messages;
pub mod threads;

pub mod marks;
pub mod tabs;

pub mod auth;
pub mod authors;
pub mod files;
pub mod games;
pub mod uploads;
pub mod users;

pub trait FromReport {
    fn from_report(report: eyre::Report) -> Self;
}

#[macro_export]
macro_rules! bail {
    ($($tt:tt)*) => {
        return ::core::result::Result::Err($crate::report!($($tt)*))
    };
}

#[macro_export]
macro_rules! report {
    ($($tt:tt)*) => {
        $crate::errors::FromReport::from_report($crate::_priv::eyre::eyre!($($tt)*))
    };
}

#[macro_export]
macro_rules! mk_error {
    ($ident:ident) => {
        #[derive(Debug)]
        #[repr(transparent)]
        pub struct $ident($crate::_priv::eyre::Report);

        const _: () = {
            use std::error::Error as StdError;
            use $crate::{_priv::eyre, errors::FromReport};

            impl $ident {
                pub fn map_report(self, f: impl FnOnce(eyre::Report) -> eyre::Report) -> Self {
                    Self(f(self.0))
                }

                pub const fn from_report(report: eyre::Report) -> Self {
                    Self(report)
                }
            }

            impl FromReport for $ident {
                fn from_report(report: eyre::Report) -> Self {
                    Self(report)
                }
            }

            impl From<$ident> for eyre::Report {
                fn from(value: $ident) -> Self {
                    value.0
                }
            }

            impl<E: StdError + Send + Sync + 'static> From<E> for $ident {
                #[track_caller]
                fn from(value: E) -> Self {
                    Self(value.into())
                }
            }
        };
    };
}

pub type Result<O, E> = ::core::result::Result<O, Generic<E>>;
pub type AuxResult<O> = ::core::result::Result<O, Aux>;

#[data(copy, ord, display("impossible"), error)]
#[derive(Hash)]
pub enum Impossible {}

/// Error from auxiliary system.
#[data(error)]
pub enum Aux {
    /// Database failure.
    #[display("{_0}")]
    Db(CompactString),
    /// Captcha failure.
    #[display("{_0}")]
    Captcha(CompactString),
    /// Mail failure.
    #[display("{_0}")]
    Mail(CompactString),
    /// Authentication failure.
    #[display("unauthenticated")]
    Unauthenticated,
    #[display("{_0}")]
    InvalidSession(#[from] auth::InvalidSession),
    /// Authorization failure.
    #[display("{_0}")]
    InvalidRole(#[from] auth::InvalidRole),
    /// Internal failure.
    #[display("{_0}")]
    InternalError(String),
    /// object store failure.
    #[display("{_0}")]
    ObjectStore(CompactString),
    #[display("{_0}")]
    Deserialization(String),
}

/// Generic error.
#[data]
#[serde(untagged)]
pub enum Generic<S> {
    /// Auxiliary system failure.
    Aux(#[from] Aux),
    /// Specific for this handler error.
    Spec(S),
}

impl<S> From<Impossible> for Generic<S> {
    fn from(value: Impossible) -> Self {
        match value {}
    }
}

// TODO: add ability to specify bounds for error.
impl<S: fmt::Display + std::error::Error> std::error::Error for Generic<S> {}

// TODO: add ability to specify bounds in display attr.
impl<S: fmt::Display> fmt::Display for Generic<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aux(aux) => aux.fmt(f),
            Self::Spec(spec) => spec.fmt(f),
        }
    }
}
