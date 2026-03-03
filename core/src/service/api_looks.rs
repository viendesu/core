use crate::service::IsSession;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Session<S>(S);

impl<S: IsSession> Session<S> {
    pub const fn new(session: S) -> Self {
        Self(session)
    }
}

macro_rules! project {
    ($(fn $method:ident() -> $($path:ident)::+;)*) => {$(
        pub const fn $method(&mut self) -> impl $crate::service::$($path)::* {
            &mut self.0
        }
    )*};
}

#[rustfmt::skip]
impl<S: IsSession> Session<S> {project!{
    fn users() -> users::Users;
    fn authors() -> authors::Authors;
    fn games() -> games::Games;

    fn boards() -> boards::Boards;
    fn threads() -> threads::Threads;
    fn messages() -> messages::Messages;

    fn authz() -> authz::Authentication;

    fn tags() -> marks::Tags;
    fn genres() -> marks::Genres;
    fn badges() -> marks::Badges;

    fn tabs() -> tabs::Tabs;

    fn uploads() -> uploads::Uploads;
}}
