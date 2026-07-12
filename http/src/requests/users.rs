use eva::data;

use viendesu_protocol::{errors, requests::users as reqs};

use super::status_code;

impl_req!(reqs::search::Args => [reqs::search::Ok; reqs::search::Err]);

status_code::direct!(reqs::search::Ok => OK);
status_code::map!(reqs::search::Err => []);

impl_req!(reqs::begin_auth::Args => [reqs::begin_auth::Ok; reqs::begin_auth::Err]);
status_code::direct!(reqs::begin_auth::Ok => OK);
status_code::map!(reqs::begin_auth::Err => []);

#[data]
pub struct FinishAuth {}

impl_req!(FinishAuth => [reqs::finish_auth::Ok; reqs::finish_auth::Err]);
status_code::direct!(reqs::finish_auth::Ok => OK);
status_code::map!(reqs::finish_auth::Err => [NoSuchAuthSession]);

impl_req!(reqs::check_auth::Args => [reqs::check_auth::Ok; reqs::check_auth::Err]);
status_code::direct!(reqs::check_auth::Ok => OK);
status_code::map!(reqs::check_auth::Err => []);

#[data]
#[non_exhaustive]
#[derive(Default)]
pub struct Get {}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);
status_code::map!(reqs::get::Err => [NotFound]);
status_code::direct!(reqs::get::Ok => OK);

impl_req!(reqs::sign_in::Args => [reqs::sign_in::Ok; reqs::sign_in::Err]);
status_code::map!(reqs::sign_in::Err => [NotFound, InvalidPassword, MustCompleteSignUp]);
status_code::direct!(reqs::sign_in::Ok => OK);

#[data]
#[non_exhaustive]
#[derive(Default)]
pub struct ConfirmSignUp {}

impl_req!(ConfirmSignUp => [reqs::confirm_sign_up::Ok; reqs::confirm_sign_up::Err]);
status_code::direct!(reqs::confirm_sign_up::Ok => OK);
status_code::map!(reqs::confirm_sign_up::Err => [NotFoundOrCompleted, InvalidSignUpToken]);

impl_req!(reqs::sign_up::Args => [reqs::sign_up::Ok; reqs::sign_up::Err]);
status_code::direct!(reqs::sign_up::Ok => CREATED);
status_code::map!(reqs::sign_up::Err => [AlreadyTaken, SignUpDisabled]);

impl_req!(reqs::update::Update => [reqs::update::Ok; reqs::update::Err]);
status_code::direct!(reqs::update::Ok => OK);
status_code::map!(reqs::update::Err => [NotFound]);

const _: () = {
    use errors::users::*;
    use status_code::direct;

    direct!(NotFound => NOT_FOUND);
    direct!(InvalidPassword => FORBIDDEN);
    direct!(MustCompleteSignUp => FORBIDDEN);
    direct!(AlreadyTaken => BAD_REQUEST);
    direct!(SignUpDisabled => FORBIDDEN);
    direct!(NotFoundOrCompleted => BAD_REQUEST);
    direct!(InvalidSignUpToken => BAD_REQUEST);
    direct!(NoSuchAuthSession => NOT_FOUND);
};
