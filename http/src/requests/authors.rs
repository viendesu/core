use eva::{data, str::CompactString};

use viendesu_core::{
    errors,
    requests::authors as reqs,
    types::{Patch, author, file, user},
};

use crate::requests::status_code;

#[data]
pub struct Search {
    pub query: CompactString,
    pub owned_by: Option<user::Selector>,
    pub limit: Option<reqs::search::Limit>,
    pub start_from: Option<author::Id>,
}

impl_req!(Search => [reqs::search::Ok; reqs::search::Err]);

status_code::direct!(reqs::search::Ok => OK);
status_code::map!(reqs::search::Err => [NoSuchUser]);

#[serde_with::apply(Patch => #[serde(default)])]
#[data]
pub struct Update {
    pub title: Patch<author::Title>,
    pub description: Patch<Option<author::Description>>,
    pub pfp: Patch<Option<file::Id>>,
    pub slug: Patch<author::Slug>,
    pub verified: Patch<bool>,
}

impl_req!(Update => [reqs::update::Ok; reqs::update::Err]);

status_code::direct!(reqs::update::Ok => OK);
status_code::map!(reqs::update::Err => [NotFound]);

#[data]
pub struct Create {
    pub title: author::Title,
    pub slug: author::Slug,
    pub pfp: Option<file::Id>,
    pub description: Option<author::Description>,
    pub owner: Option<user::Id>,
}

impl_req!(Create => [reqs::create::Ok; reqs::create::Err]);

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
