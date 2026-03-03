use eva::auto_impl;

use crate::{
    requests::users::{
        begin_auth, check_auth, confirm_sign_up, finish_auth, get, sign_in, sign_up, update,
    },
    service::CallStep,
};

#[auto_impl(&mut)]
pub trait Users: Send + Sync {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err>;
    fn check_auth(
        &mut self,
    ) -> impl CallStep<check_auth::Args, Ok = check_auth::Ok, Err = check_auth::Err>;

    fn begin_auth(
        &mut self,
    ) -> impl CallStep<begin_auth::Args, Ok = begin_auth::Ok, Err = begin_auth::Err>;
    fn finish_auth(
        &mut self,
    ) -> impl CallStep<finish_auth::Args, Ok = finish_auth::Ok, Err = finish_auth::Err>;

    fn sign_in(&mut self) -> impl CallStep<sign_in::Args, Ok = sign_in::Ok, Err = sign_in::Err>;
    fn sign_up(&mut self) -> impl CallStep<sign_up::Args, Ok = sign_up::Ok, Err = sign_up::Err>;

    fn update(&mut self) -> impl CallStep<update::Args, Ok = update::Ok, Err = update::Err>;

    fn confirm_sign_up(
        &mut self,
    ) -> impl CallStep<confirm_sign_up::Args, Ok = confirm_sign_up::Ok, Err = confirm_sign_up::Err>;
}
