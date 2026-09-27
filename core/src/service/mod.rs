use eva::{auto_impl, trait_set};

use viendesu_protocol::errors::Aux;

pub use eva::handling::CallStep;

viendesu_macros::endpoints!(for_each_endpoint);

/// Generates the [`IsSession`] trait sum and the [`Session`] projection methods.
macro_rules! domains {
    ($(
        $segment:ident {$(
            $accessor:ident : $Trait:ident {$(
                $(#[$flag:ident])*
                $method:ident = $endpoint:ident {
                    args: $Args:ty,
                    ok: $Ok:ty,
                    err: $Err:ty,
                    output: $Output:ty,
                }
            )*}
        )*}
    )*) => {
        trait_set! {
            pub trait IsSession = Send + Sync + authz::Authentication $($(+ $segment::$Trait)*)*;
        }

        impl<S: IsSession> Session<S> {
            pub const fn authz(&mut self) -> impl authz::Authentication {
                &mut self.0
            }

            $($(
                pub const fn $accessor(&mut self) -> impl $segment::$Trait {
                    &mut self.0
                }
            )*)*
        }
    };
}

for_each_endpoint!(domains);

pub mod files;

pub mod authz;

pub type SessionOf<T> = <T as SessionMaker>::Session;

trait_set! {
    pub trait AuxFut<O> = Future<Output = Result<O, Aux>> + Send;

    pub trait IsService = SessionMaker;
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Session<S>(S);

impl<S: IsSession> Session<S> {
    pub const fn new(session: S) -> Self {
        Self(session)
    }
}

#[auto_impl(&, &mut, Arc)]
pub trait SessionMaker: Send + Sync {
    type Session: IsSession;

    fn make_session(&self) -> impl AuxFut<Session<Self::Session>>;
}
