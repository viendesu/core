use eva::data;

use crate::requests::status_code;

use viendesu_core::{
    errors,
    requests::boards as reqs,
    types::{Patch, board, message},
};

#[data]
pub struct Get {}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);

status_code::direct!(reqs::get::Ok => OK);
status_code::map!(reqs::get::Err => [NotFound]);

#[data]
pub struct Edit {
    pub text: Patch<message::Text>,
    pub slug: Patch<Option<board::Slug>>,
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
pub struct Create {
    pub slug: board::Slug,
    pub initial_message: message::Text,
    pub by: Option<message::ById>,
}

impl_req!(Create => [reqs::create::Ok; reqs::create::Err]);

status_code::direct!(reqs::create::Ok => OK);
status_code::map!(reqs::create::Err => [AlreadyExists]);

const _: () = {
    use errors::boards::*;
    use status_code::direct;

    direct!(AlreadyExists => BAD_REQUEST);
    direct!(NotFound => NOT_FOUND);
};
