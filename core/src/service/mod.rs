use eva::{auto_impl, handling, trait_set};

use crate::{errors::Aux, requests::Response};

pub use self::api_looks::Session;

mod api_looks;

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
    pub trait IsSession = boards::Boards
        + messages::Messages
        + threads::Threads
        + authors::Authors
        + games::Games
        + users::Users
        + authz::Authentication
        + marks::Tags
        + marks::Genres
        + marks::Badges
        + tabs::Tabs
        + uploads::Uploads
    ;
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
