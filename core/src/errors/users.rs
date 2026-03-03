use eva::data;

use crate::types::user;

#[data(error, display("there is no such auth session"))]
pub struct NoSuchAuthSession;

#[data(copy, display(name))]
pub enum WhatTaken {
    Nickname,
    Email,
}

#[data(error, display("passed invalid sign up token"))]
pub struct InvalidSignUpToken;

#[data(error, display("the {what} is already taken"))]
pub struct AlreadyTaken {
    pub what: WhatTaken,
}

#[data(
    error,
    display("sign up for specified user was already completed or the user was not found")
)]
pub struct NotFoundOrCompleted;

#[data(error, display("user {user} was not found"))]
pub struct NotFound {
    pub user: user::Selector,
}

#[data(error, display("invalid password specified"))]
pub struct InvalidPassword;

#[data(error, display("the session is required for this request"))]
pub struct NoSession;

#[data(error, display("you must complete sign up first"))]
pub struct MustCompleteSignUp;
