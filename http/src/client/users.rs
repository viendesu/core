use super::*;

use viendesu_core::requests::users::{
    begin_auth, check_auth, confirm_sign_up, finish_auth, get, search, sign_in, sign_up, update,
};

use crate::requests::users as requests;

impl Users for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(Method::GET, |get::Args { user }| match user {
            Some(u) => (c!("/users/{u}"), requests::Get {}),
            None => ("/users/me".into(), requests::Get {}),
        })
    }

    fn search(&mut self) -> impl CallStep<search::Args, Ok = search::Ok, Err = search::Err> {
        self.do_call(
            Method::GET,
            |search::Args {
                 query,
                 limit,
                 start_from,
             }| {
                (
                    c!("/users"),
                    requests::Search {
                        query,
                        limit,
                        start_from,
                    },
                )
            },
        )
    }

    fn check_auth(
        &mut self,
    ) -> impl CallStep<check_auth::Args, Ok = check_auth::Ok, Err = check_auth::Err> {
        self.do_call(Method::GET, |check_auth::Args {}| {
            ("/users/check_auth".into(), requests::CheckAuth {})
        })
    }

    fn begin_auth(
        &mut self,
    ) -> impl CallStep<begin_auth::Args, Ok = begin_auth::Ok, Err = begin_auth::Err> {
        self.do_call(Method::POST, |begin_auth::Args { method }| {
            ("/users/begin-auth".into(), requests::BeginAuth { method })
        })
    }

    fn finish_auth(
        &mut self,
    ) -> impl CallStep<finish_auth::Args, Ok = finish_auth::Ok, Err = finish_auth::Err> {
        self.do_call(Method::POST, |finish_auth::Args { auth_session }| {
            (
                c!("/users/finish-auth/{auth_session}"),
                requests::FinishAuth {},
            )
        })
    }

    fn confirm_sign_up(
        &mut self,
    ) -> impl CallStep<confirm_sign_up::Args, Ok = confirm_sign_up::Ok, Err = confirm_sign_up::Err>
    {
        self.do_call(Method::POST, todo::<_, requests::ConfirmSignUp>())
    }

    fn update(&mut self) -> impl CallStep<update::Args, Ok = update::Ok, Err = update::Err> {
        self.do_call(Method::PATCH, |update::Args { user, update }| match user {
            Some(u) => (c!("/users/{u}"), requests::Update::from(update)),
            None => ("/users/me".into(), requests::Update::from(update)),
        })
    }

    fn sign_up(&mut self) -> impl CallStep<sign_up::Args, Ok = sign_up::Ok, Err = sign_up::Err> {
        self.do_call(
            Method::POST,
            |sign_up::Args {
                 nickname,
                 email,
                 display_name,
                 password,
             }| {
                (
                    "/users/sign_up".into(),
                    requests::SignUp {
                        nickname,
                        email,
                        password,
                        display_name,
                    },
                )
            },
        )
    }

    fn sign_in(&mut self) -> impl CallStep<sign_in::Args, Ok = sign_in::Ok, Err = sign_in::Err> {
        self.do_call(Method::POST, |sign_in::Args { nickname, password }| {
            (
                "/users/sign_in".into(),
                requests::SignIn { nickname, password },
            )
        })
    }
}
