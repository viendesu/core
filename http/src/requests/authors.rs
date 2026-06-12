use eva::data;

use viendesu_protocol::{errors, requests::authors as reqs};

use crate::requests::status_code;

impl_req!(reqs::search::Args => [reqs::search::Ok; reqs::search::Err]);

status_code::direct!(reqs::search::Ok => OK);
status_code::map!(reqs::search::Err => [NoSuchUser]);

impl_req!(reqs::update::Update => [reqs::update::Ok; reqs::update::Err]);

status_code::direct!(reqs::update::Ok => OK);
status_code::map!(reqs::update::Err => [NotFound]);

impl_req!(reqs::create::Args => [reqs::create::Ok; reqs::create::Err]);

status_code::direct!(reqs::create::Ok => OK);
status_code::map!(reqs::create::Err => [NotFound, AlreadyExists, NoSuchUser]);

#[data]
pub struct Get {}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);

status_code::direct!(reqs::get::Ok => OK);
status_code::map!(reqs::get::Err => [NotFound]);

const _: () = {
    use errors::authors::*;
    use status_code::direct;

    direct!(NotFound => NOT_FOUND);
    direct!(AlreadyExists => BAD_REQUEST);
};
