use eva::{data, str::CompactString};

use viendesu_core::{
    errors,
    requests::users as reqs,
    types::{Patch, file, user as core},
};

use super::status_code;

#[data]
pub struct Search {
    pub query: Option<CompactString>,
    #[serde(default)]
    pub limit: reqs::search::Limit,
    pub start_from: Option<core::Id>,
}

impl_req!(Search => [reqs::search::Ok; reqs::search::Err]);

status_code::direct!(reqs::search::Ok => OK);
status_code::map!(reqs::search::Err => []);

#[data]
pub struct BeginAuth {
    pub method: core::AuthoredAuth,
}

impl_req!(BeginAuth => [reqs::begin_auth::Ok; reqs::begin_auth::Err]);
status_code::direct!(reqs::begin_auth::Ok => OK);
status_code::map!(reqs::begin_auth::Err => []);

#[data]
pub struct FinishAuth {}

impl_req!(FinishAuth => [reqs::finish_auth::Ok; reqs::finish_auth::Err]);
status_code::direct!(reqs::finish_auth::Ok => OK);
status_code::map!(reqs::finish_auth::Err => [NoSuchAuthSession]);

#[data]
pub struct CheckAuth {}

impl_req!(CheckAuth => [reqs::check_auth::Ok; reqs::check_auth::Err]);
status_code::direct!(reqs::check_auth::Ok => OK);
status_code::map!(reqs::check_auth::Err => []);

#[data]
#[non_exhaustive]
#[derive(Default)]
pub struct Get {}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);
status_code::map!(reqs::get::Err => [NotFound]);
status_code::direct!(reqs::get::Ok => OK);

#[data]
pub struct SignIn {
    pub nickname: core::Nickname,
    pub password: core::Password,
}

impl_req!(SignIn => [reqs::sign_in::Ok; reqs::sign_in::Err]);
status_code::map!(reqs::sign_in::Err => [NotFound, InvalidPassword, MustCompleteSignUp]);
status_code::direct!(reqs::sign_in::Ok => OK);

#[data]
#[non_exhaustive]
#[derive(Default)]
pub struct ConfirmSignUp {}

impl_req!(ConfirmSignUp => [reqs::confirm_sign_up::Ok; reqs::confirm_sign_up::Err]);
status_code::direct!(reqs::confirm_sign_up::Ok => OK);
status_code::map!(reqs::confirm_sign_up::Err => [NotFoundOrCompleted, InvalidSignUpToken]);

#[data]
pub struct SignUp {
    pub nickname: core::Nickname,
    pub email: core::Email,
    pub password: core::Password,
    pub display_name: Option<core::DisplayName>,
}

impl_req!(SignUp => [reqs::sign_up::Ok; reqs::sign_up::Err]);
status_code::direct!(reqs::sign_up::Ok => CREATED);
status_code::map!(reqs::sign_up::Err => [AlreadyTaken]);

#[serde_with::apply(
    Patch => #[serde(default)]
)]
#[data]
#[non_exhaustive]
#[derive(Default)]
pub struct Update {
    pub nickname: Patch<core::Nickname>,
    pub display_name: Patch<Option<core::DisplayName>>,
    pub bio: Patch<Option<core::Bio>>,
    pub password: Patch<core::Password>,
    pub role: Patch<core::Role>,
    pub pfp: Patch<Option<file::Id>>,
    pub email: Patch<core::Email>,
}

impl From<reqs::update::Update> for Update {
    fn from(u: reqs::update::Update) -> Self {
        Self {
            nickname: u.nickname,
            display_name: u.display_name,
            bio: u.bio,
            password: u.password,
            role: u.role,
            pfp: u.pfp,
            email: u.email,
        }
    }
}

impl_req!(Update => [reqs::update::Ok; reqs::update::Err]);
status_code::direct!(reqs::update::Ok => OK);
status_code::map!(reqs::update::Err => [NotFound]);

const _: () = {
    use errors::users::*;
    use status_code::direct;

    direct!(NotFound => NOT_FOUND);
    direct!(InvalidPassword => FORBIDDEN);
    direct!(MustCompleteSignUp => FORBIDDEN);
    direct!(AlreadyTaken => BAD_REQUEST);
    direct!(NotFoundOrCompleted => BAD_REQUEST);
    direct!(InvalidSignUpToken => BAD_REQUEST);
    direct!(NoSuchAuthSession => NOT_FOUND);
};
