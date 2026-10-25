pub use eva::handling::CallStep;
use eva::{auto_impl, export, trait_set};
use viendesu_protocol::errors::Aux;
pub mod authz;

pub use crate::protocol::requests::articles::Articles;
pub use crate::protocol::requests::authors::Authors;
pub use crate::protocol::requests::badges::Badges;
pub use crate::protocol::requests::blogs::Blogs;
pub use crate::protocol::requests::boards::Boards;
pub use crate::protocol::requests::comments::Comments;
pub use crate::protocol::requests::files::Files;
pub use crate::protocol::requests::games::Games;
pub use crate::protocol::requests::genres::Genres;
pub use crate::protocol::requests::messages::Messages;
pub use crate::protocol::requests::tabs::Tabs;
pub use crate::protocol::requests::tags::Tags;
pub use crate::protocol::requests::threads::Threads;
pub use crate::protocol::requests::uploads::Uploads;
pub use crate::protocol::requests::users::Users;
#[export]
#[derive(Debug, Clone, Copy)]
pub struct Session {
    users: crate::protocol::requests::users::Users,
    authors: crate::protocol::requests::authors::Authors,
    games: crate::protocol::requests::games::Games,
    boards: crate::protocol::requests::boards::Boards,
    threads: crate::protocol::requests::threads::Threads,
    messages: crate::protocol::requests::messages::Messages,
    blogs: crate::protocol::requests::blogs::Blogs,
    articles: crate::protocol::requests::articles::Articles,
    comments: crate::protocol::requests::comments::Comments,
    tags: crate::protocol::requests::tags::Tags,
    genres: crate::protocol::requests::genres::Genres,
    badges: crate::protocol::requests::badges::Badges,
    tabs: crate::protocol::requests::tabs::Tabs,
    uploads: crate::protocol::requests::uploads::Uploads,
}

trait_set! {
    pub trait IsSession = SessionFeatures + authz::Authentication;
    pub trait AuxFut<O> = Future<Output = Result<O, Aux>> + Send;
    pub trait IsService = SessionMaker;
}
pub type SessionOf<T> = <T as SessionMaker>::Session;
impl<S: IsSession> Session<S> {
    pub fn authz(&mut self) -> impl authz::Authentication {
        &mut self.0
    }
}
#[auto_impl(&, &mut, Arc)]
pub trait SessionMaker: Send + Sync {
    type Session: IsSession;
    fn make_session(&self) -> impl AuxFut<Session<Self::Session>>;
}
