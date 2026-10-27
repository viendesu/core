use eva::{auto_impl, export, trait_set};
use viendesu_protocol::errors::Aux;

use crate::protocol::requests::{
    Articles, Authors, Badges, Blogs, Boards, Comments, Games, Genres, Messages, Tabs, Tags,
    Threads, Uploads, Users,
};

pub mod authz;

#[export(flags(write, stream))]
#[derive(Debug, Clone, Copy)]
pub struct Session {
    users: Users,
    authors: Authors,
    games: Games,
    boards: Boards,
    threads: Threads,
    messages: Messages,
    blogs: Blogs,
    articles: Articles,
    comments: Comments,
    tags: Tags,
    genres: Genres,
    badges: Badges,
    tabs: Tabs,
    uploads: Uploads,
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
