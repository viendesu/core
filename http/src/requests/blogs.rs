use eva::data;

use crate::requests::status_code;

use viendesu_protocol::{
    errors,
    requests::blogs as reqs,
    types::{Patch, blog},
};

#[data]
pub struct Get {}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);

status_code::direct!(reqs::get::Ok => OK);
status_code::map!(reqs::get::Err => [NotFound, NoSuchUser, NoSuchAuthor, NoSuchGame]);

#[data]
pub struct Edit {
    #[serde(default)]
    pub title: Patch<Option<blog::Title>>,
    #[serde(default)]
    pub description: Patch<Option<blog::Description>>,
}

impl_req!(Edit => [reqs::edit::Ok; reqs::edit::Err]);

status_code::direct!(reqs::edit::Ok => OK);
status_code::map!(reqs::edit::Err => [NotFound, NotAnOwner]);

const _: () = {
    use errors::blogs::*;
    use status_code::direct;

    direct!(NotAnOwner => FORBIDDEN);
    direct!(NotFound => NOT_FOUND);
};
