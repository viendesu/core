use eva::data;

use crate::requests::status_code;

use viendesu_protocol::{errors, requests::comments as reqs, types::comment};

#[data]
pub struct List {
    #[serde(default)]
    pub sort: comment::Sort,
    pub before: Option<comment::Id>,
    pub before_likes: Option<comment::Likes>,
    #[serde(default)]
    pub limit: comment::Limit,
}

impl_req!(List => [reqs::list::Ok; reqs::list::Err]);

status_code::direct!(reqs::list::Ok => OK);
status_code::map!(reqs::list::Err => [NoSuchGame]);

#[data]
pub struct Replies {
    pub after: Option<comment::Id>,
    #[serde(default)]
    pub limit: comment::Limit,
}

impl_req!(Replies => [reqs::replies::Ok; reqs::replies::Err]);

status_code::direct!(reqs::replies::Ok => OK);
status_code::map!(reqs::replies::Err => [NotFound]);

#[data]
pub struct Create {
    pub text: comment::Text,
    #[serde(default)]
    pub parent: Option<comment::Id>,
}

impl_req!(Create => [reqs::create::Ok; reqs::create::Err]);

status_code::direct!(reqs::create::Ok => CREATED);
status_code::map!(reqs::create::Err => [NoSuchGame, NoSuchParent, InvalidParent]);

#[data]
pub struct Edit {
    pub text: comment::Text,
}

impl_req!(Edit => [reqs::edit::Ok; reqs::edit::Err]);

status_code::direct!(reqs::edit::Ok => OK);
status_code::map!(reqs::edit::Err => [NotFound]);

#[data]
pub struct Delete {}

impl_req!(Delete => [reqs::delete::Ok; reqs::delete::Err]);

status_code::direct!(reqs::delete::Ok => OK);
status_code::map!(reqs::delete::Err => [NotFound]);

#[data]
pub struct Like {
    /// `false` retracts the caller's like.
    pub liked: bool,
}

impl_req!(Like => [reqs::like::Ok; reqs::like::Err]);

status_code::direct!(reqs::like::Ok => OK);
status_code::map!(reqs::like::Err => [NotFound]);

const _: () = {
    use errors::comments::*;
    use status_code::direct;

    direct!(NotFound => NOT_FOUND);
    direct!(InvalidParent => CONFLICT);
};
