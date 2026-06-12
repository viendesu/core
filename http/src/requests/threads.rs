use eva::data;

use crate::requests::status_code;

use viendesu_protocol::{
    errors,
    requests::threads as reqs,
    types::{Patch, message},
};

#[data]
pub struct Get {}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);

status_code::direct!(reqs::get::Ok => OK);
status_code::map!(reqs::get::Err => [NotFound]);

#[data]
pub struct Delete {}

impl_req!(Delete => [reqs::delete::Ok; reqs::delete::Err]);

status_code::direct!(reqs::delete::Ok => OK);
status_code::map!(reqs::delete::Err => [NotFound]);

#[data]
pub struct Edit {
    pub text: Patch<message::Text>,
}

impl_req!(Edit => [reqs::edit::Ok; reqs::edit::Err]);

status_code::direct!(reqs::edit::Ok => OK);
status_code::map!(reqs::edit::Err => [NotFound, NotAnOwner]);

impl_req!(reqs::search::Args => [reqs::search::Ok; reqs::search::Err]);

status_code::direct!(reqs::search::Ok => OK);
status_code::map!(reqs::search::Err => []);

impl_req!(reqs::create::Args => [reqs::create::Ok; reqs::create::Err]);

status_code::direct!(reqs::create::Ok => CREATED);
status_code::map!(reqs::create::Err => [NoSuchBoard]);

const _: () = {
    use errors::threads::*;
    use status_code::direct;

    direct!(NotAnOwner => FORBIDDEN);
    direct!(NotFound => NOT_FOUND);
};
