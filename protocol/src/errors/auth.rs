use eva::data;

use crate::types::{session, user::Role};

#[data(
    error,
    display(
        "invalid role, it's required to be at least {required_at_least} to perform this action"
    )
)]
pub struct InvalidRole {
    pub required_at_least: Role,
}

#[data(error, display("session {token} does not exists or is expired"))]
pub struct InvalidSession {
    pub token: session::Token,
}
