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

/// The single list of service endpoints: invokes `$callback! { ... }` with it.
///
/// Every group `module { accessor: Trait { method, other => endpoint } }` names
/// the protocol module `viendesu_protocol::requests::module` and the service
/// module `viendesu_core::service::module`; `accessor` is the [`Session`]
/// projection of `Trait`, and the endpoint module defaults to the method name.
/// `#[stream]` marks endpoints whose `Args` are not serde types.
///
/// Callbacks must accept every group, so match the full grammar:
///
/// ```ignore
/// $( $module:ident { $( $accessor:ident : $Trait:ident {
///     $( $(#[$attr:ident])* $method:ident $(=> $endpoint:ident)? ),* $(,)?
/// } )* } )*
/// ```
#[macro_export]
macro_rules! for_each_endpoint {
    ($callback:ident) => {
        $callback! {
            users {
                users: Users {
                    get,
                    check_auth,
                    search,
                    begin_auth,
                    finish_auth,
                    sign_in,
                    sign_up,
                    update,
                    confirm_sign_up,
                }
            }
            authors {
                authors: Authors { get, search, create, update }
            }
            games {
                games: Games { get, search, create, update, rate }
            }

            boards {
                boards: Boards { get, create, delete, edit }
            }
            threads {
                threads: Threads { get, search, delete, edit, create }
            }
            messages {
                messages: Messages { get, post, delete, edit }
            }

            blogs {
                blogs: Blogs { get, edit }
            }
            articles {
                articles: Articles { get, search, create, delete, edit }
            }
            comments {
                comments: Comments { list, replies, create, edit, delete, like }
            }

            marks {
                tags: Tags { list => list_tags, add => add_tag }
                genres: Genres { list => list_genres }
                badges: Badges { list => list_badges, add => add_badge }
            }
            tabs {
                tabs: Tabs { list, insert, delete, list_items }
            }

            uploads {
                uploads: Uploads { list_pending, start, abort, #[stream] finish }
            }
        }
    };
}

macro_rules! define_services {
    ($(
        $module:ident {$(
            $accessor:ident : $Trait:ident {
                $( $(#[$attr:ident])* $method:ident $(=> $endpoint:ident)? ),* $(,)?
            }
        )*}
    )*) => {
        $(
            pub mod $module {$(
                service_trait! {
                    pub trait $Trait(viendesu_protocol::requests::$module) {
                        $( $method $(=> $endpoint)? ),*
                    }
                }
            )*}
        )*

        domains! {
            $($( $accessor: $module::$Trait, )*)*
            authz: authz::Authentication,
        }
    };
}

for_each_endpoint!(define_services);

pub mod files;

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
