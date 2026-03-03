use eva::{data, str::CompactString};

use crate::requests::status_code;

use viendesu_core::requests::marks as reqs;

#[data]
pub struct AddBadge {
    pub text: CompactString,
}

impl_req!(AddBadge => [reqs::add_badge::Ok; reqs::add_badge::Err]);
status_code::direct!(reqs::add_badge::Ok => CREATED);
status_code::map!(reqs::add_badge::Err => []);

#[data]
pub struct AddTag {
    pub text: CompactString,
}

impl_req!(AddTag => [reqs::add_tag::Ok; reqs::add_tag::Err]);
status_code::direct!(reqs::add_tag::Ok => CREATED);
status_code::map!(reqs::add_tag::Err => []);

#[data]
pub struct ListGenres {}

impl_req!(ListGenres => [reqs::list_genres::Ok; reqs::list_genres::Err]);
status_code::direct!(reqs::list_genres::Ok => OK);
status_code::map!(reqs::list_genres::Err => []);

#[data]
pub struct ListTags {
    pub query: Option<CompactString>,
}

impl_req!(ListTags => [reqs::list_tags::Ok; reqs::list_tags::Err]);
status_code::direct!(reqs::list_tags::Ok => OK);
status_code::map!(reqs::list_tags::Err => []);

#[data]
pub struct ListBadges {
    pub query: Option<CompactString>,
}

impl_req!(ListBadges => [reqs::list_badges::Ok; reqs::list_badges::Err]);
status_code::direct!(reqs::list_badges::Ok => OK);
status_code::map!(reqs::list_badges::Err => []);

const _: () = {
    use status_code::*;
    use viendesu_core::errors::marks::*;

    direct!(NoSuchTag => NOT_FOUND);
    direct!(NoSuchGenre => NOT_FOUND);
    direct!(NoSuchBadge => NOT_FOUND);
};
