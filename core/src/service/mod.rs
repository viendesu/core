use eva::{auto_impl, handling, trait_set};

use viendesu_protocol::{errors::Aux, requests::Response};

/// Generates a service domain trait from a list of endpoints.
///
/// Each `method` entry expands to
/// `fn method(&mut self) -> impl CallStep<module::Args, Ok = module::Ok, Err = module::Err>`
/// against the given protocol requests module. The endpoint module defaults
/// to the method name; `method => module` overrides it.
macro_rules! service_trait {
    (
        $(#[$meta:meta])*
        pub trait $Trait:ident($($reqs:ident)::+) {
            $($body:tt)*
        }
    ) => {
        service_trait!(@parse [$(#[$meta])*] $Trait [$($reqs)::+] [] $($body)*);
    };

    (@parse $meta:tt $Trait:ident $reqs:tt [$($acc:tt)*] $method:ident => $module:ident, $($rest:tt)*) => {
        service_trait!(@parse $meta $Trait $reqs [$($acc)* [$method $module $reqs]] $($rest)*);
    };
    (@parse $meta:tt $Trait:ident $reqs:tt [$($acc:tt)*] $method:ident => $module:ident) => {
        service_trait!(@parse $meta $Trait $reqs [$($acc)* [$method $module $reqs]]);
    };
    (@parse $meta:tt $Trait:ident $reqs:tt [$($acc:tt)*] $method:ident, $($rest:tt)*) => {
        service_trait!(@parse $meta $Trait $reqs [$($acc)* [$method $method $reqs]] $($rest)*);
    };
    (@parse $meta:tt $Trait:ident $reqs:tt [$($acc:tt)*] $method:ident) => {
        service_trait!(@parse $meta $Trait $reqs [$($acc)* [$method $method $reqs]]);
    };

    (@parse [$($meta:tt)*] $Trait:ident $reqs:tt [$([$method:ident $module:ident [$($path:ident)::+]])*]) => {
        $($meta)*
        #[::eva::auto_impl(&mut, Box)]
        pub trait $Trait: Send + Sync {$(
            fn $method(
                &mut self,
            ) -> impl $crate::service::CallStep<
                $($path)::+::$module::Args,
                Ok = $($path)::+::$module::Ok,
                Err = $($path)::+::$module::Err,
            >;
        )*}
    };
}

/// Registers the session domains: generates the [`IsSession`] trait sum
/// and the [`Session`] projection methods from a single list.
macro_rules! domains {
    ($($method:ident: $($Trait:ident)::+),* $(,)?) => {
        trait_set! {
            pub trait IsSession = Send + Sync $(+ $($Trait)::+)*;
        }

        impl<S: IsSession> Session<S> {$(
            pub const fn $method(&mut self) -> impl $($Trait)::+ {
                &mut self.0
            }
        )*}
    };
}

pub mod marks;
pub mod tabs;

pub mod boards;
pub mod messages;
pub mod threads;

pub mod authors;
pub mod games;
pub mod users;

pub mod files;
pub mod uploads;

pub mod authz;

pub type SessionOf<T> = <T as SessionMaker>::Session;

trait_set! {
    pub trait RespFut<O, E> = Future<Output = Response<O, E>> + Send;
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

domains! {
    users: users::Users,
    authors: authors::Authors,
    games: games::Games,

    boards: boards::Boards,
    threads: threads::Threads,
    messages: messages::Messages,

    authz: authz::Authentication,

    tags: marks::Tags,
    genres: marks::Genres,
    badges: marks::Badges,

    tabs: tabs::Tabs,

    uploads: uploads::Uploads,
}

#[auto_impl(&, &mut, Arc)]
pub trait SessionMaker: Send + Sync {
    type Session: IsSession;

    fn make_session(&self) -> impl AuxFut<Session<Self::Session>>;
}

pub trait CallStep<I>: Send + Sync {
    type Ok;
    type Err;

    fn call(&mut self, args: I) -> impl RespFut<Self::Ok, Self::Err>;
}

pub trait IsEndpoint<I, S>:
    handling::Endpoint<I, S, Output = Response<Self::Ok, Self::Err>>
{
    type Ok;
    type Err;
}

trait_set! {
    pub trait IsState = Send + Sync;
}

impl<I, S, O, E, Ep> IsEndpoint<I, S> for Ep
where
    Ep: handling::Endpoint<I, S, Output = Response<O, E>>,
{
    type Ok = O;
    type Err = E;
}
