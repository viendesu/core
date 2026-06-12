use super::*;

use viendesu_protocol::requests::users::{
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
        self.do_call(Method::GET, |args: search::Args| (c!("/users"), args))
    }

    fn check_auth(
        &mut self,
    ) -> impl CallStep<check_auth::Args, Ok = check_auth::Ok, Err = check_auth::Err> {
        self.do_call(Method::GET, |args: check_auth::Args| {
            ("/users/check-auth".into(), args)
        })
    }

    fn begin_auth(
        &mut self,
    ) -> impl CallStep<begin_auth::Args, Ok = begin_auth::Ok, Err = begin_auth::Err> {
        self.do_call(Method::POST, |args: begin_auth::Args| {
            ("/users/begin-auth".into(), args)
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
            Some(u) => (c!("/users/{u}"), update),
            None => ("/users/me".into(), update),
        })
    }

    fn sign_up(&mut self) -> impl CallStep<sign_up::Args, Ok = sign_up::Ok, Err = sign_up::Err> {
        self.do_call(Method::POST, |args: sign_up::Args| {
            ("/users/sign_up".into(), args)
        })
    }

    fn sign_in(&mut self) -> impl CallStep<sign_in::Args, Ok = sign_in::Ok, Err = sign_in::Err> {
        self.do_call(Method::POST, |args: sign_in::Args| {
            ("/users/sign_in".into(), args)
        })
    }
}
