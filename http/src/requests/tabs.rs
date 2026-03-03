use eva::data;

use viendesu_core::{errors, requests::tabs as reqs, types::entity};

use crate::requests::status_code;

#[data]
pub struct Delete {}

impl_req!(Delete => [reqs::delete::Ok; reqs::delete::Err]);
status_code::direct!(reqs::delete::Ok => OK);
status_code::map!(reqs::delete::Err => [InvalidItemId, NoSuchItem, NoSuchTab]);

#[data]
pub struct Insert {
    pub item: entity::Id,
}

impl_req!(Insert => [reqs::insert::Ok; reqs::insert::Err]);
status_code::direct!(reqs::insert::Ok => CREATED);
status_code::map!(reqs::insert::Err => [Duplicate, InvalidKind, NoSuchTab, InvalidItemId]);

#[data]
pub struct ListItems {
    #[serde(default)]
    pub resolve_marks: bool,
    #[serde(default)]
    pub start_from: Option<entity::Id>,
    #[serde(default)]
    pub limit: reqs::list_items::Limit,
}

impl_req!(ListItems => [reqs::list_items::Ok; reqs::list_items::Err]);
status_code::direct!(reqs::list_items::Ok => OK);
status_code::map!(reqs::list_items::Err => [InvalidItemId, NoSuchTab]);

#[data]
pub struct List {}

impl_req!(List => [reqs::list::Ok; reqs::list::Err]);
status_code::direct!(reqs::list::Ok => OK);
status_code::map!(reqs::list::Err => [NotFound]);

const _: () = {
    use errors::tabs::*;
    use status_code::direct;

    direct!(InvalidKind => BAD_REQUEST);
    direct!(Duplicate => BAD_REQUEST);
    direct!(InvalidItemId => BAD_REQUEST);
    direct!(NoSuchTab => NOT_FOUND);
    direct!(NoSuchItem => NOT_FOUND);
};
