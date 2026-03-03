use eva::data;

use crate::requests::status_code;

use viendesu_core::{
    errors,
    requests::messages as reqs,
    types::{message, thread},
};

#[data]
pub struct Edit {
    pub text: message::Text,
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
pub struct Post {
    pub thread: thread::Selector,
    pub text: message::Text,
}

impl_req!(Post => [reqs::post::Ok; reqs::post::Err]);

status_code::direct!(reqs::post::Ok => OK);
status_code::map!(reqs::post::Err => [NoSuchThread]);

#[data]
pub struct Get {}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);

status_code::direct!(reqs::get::Ok => OK);
status_code::map!(reqs::get::Err => [NotFound]);

const _: () = {
    use errors::messages::*;
    use status_code::direct;

    direct!(NotFound => NOT_FOUND);
};
