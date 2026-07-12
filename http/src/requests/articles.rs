use eva::data;

use crate::requests::status_code;

use viendesu_protocol::{
    errors,
    requests::articles as reqs,
    types::{Patch, article},
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
status_code::map!(reqs::delete::Err => [NotFound, NotAnOwner]);

#[data]
pub struct Edit {
    #[serde(default)]
    pub title: Patch<article::Title>,
    #[serde(default)]
    pub content: Patch<article::Content>,
}

impl_req!(Edit => [reqs::edit::Ok; reqs::edit::Err]);

status_code::direct!(reqs::edit::Ok => OK);
status_code::map!(reqs::edit::Err => [NotFound, NotAnOwner]);

#[data]
pub struct Search {
    #[serde(default)]
    pub limit: reqs::search::Limit,
    pub before: Option<article::Id>,
}

impl_req!(Search => [reqs::search::Ok; reqs::search::Err]);

status_code::direct!(reqs::search::Ok => OK);
status_code::map!(reqs::search::Err => [NoSuchBlog]);

#[data]
pub struct Create {
    pub title: article::Title,
    pub content: article::Content,
}

impl_req!(Create => [reqs::create::Ok; reqs::create::Err]);

status_code::direct!(reqs::create::Ok => CREATED);
status_code::map!(reqs::create::Err => [NoSuchBlog, NotAnOwner]);

const _: () = {
    use errors::articles::*;
    use status_code::direct;

    direct!(NotAnOwner => FORBIDDEN);
    direct!(NotFound => NOT_FOUND);
};
