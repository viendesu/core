use eva::data;

use crate::requests::status_code;

use viendesu_core::{errors, requests::files as reqs};

#[data]
pub struct GetInfo {}

impl_req!(GetInfo => [reqs::get_info::Ok; reqs::get_info::Err]);
status_code::direct!(reqs::get_info::Ok => OK);
status_code::map!(reqs::get_info::Err => [NotFound]);

const _: () = {
    use errors::files::*;
    use status_code::direct;

    direct!(NotFound => NOT_FOUND);
    direct!(InvalidImage => BAD_REQUEST);
    direct!(UnexpectedFileClass => BAD_REQUEST);
};
