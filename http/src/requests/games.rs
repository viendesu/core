use eva::data;

use crate::requests::status_code;

use viendesu_protocol::{errors, requests::games as reqs, types::game};

impl_req!(reqs::update::Update => [reqs::update::Ok; reqs::update::Err]);

status_code::direct!(reqs::update::Ok => OK);
status_code::map!(reqs::update::Err => [
    NotFound,
    AlreadyTaken,
    NoSuchBadge,
    NoSuchGenre,
    NoSuchTag,
    UnexpectedFileClass,
    InvalidImage,
    FileNotFound,
]);

impl_req!(reqs::search::Args => [reqs::search::Ok; reqs::search::Err]);

status_code::direct!(reqs::search::Ok => OK);
status_code::map!(reqs::search::Err => [NoSuchAuthor]);

#[data]
pub struct Get {
    #[serde(default)]
    pub resolve_marks: bool,
    #[serde(default)]
    pub latest_articles: bool,
    #[serde(default)]
    pub comments: bool,
}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);

status_code::direct!(reqs::get::Ok => OK);
status_code::map!(reqs::get::Err => [NotFound, NoSuchAuthor]);

#[data]
pub struct Rate {
    /// `None` retracts the caller's vote.
    #[serde(default)]
    pub rating: Option<game::RatingValue>,
}

impl_req!(Rate => [reqs::rate::Ok; reqs::rate::Err]);

status_code::direct!(reqs::rate::Ok => OK);
status_code::map!(reqs::rate::Err => [NotFound]);

impl_req!(reqs::create::Args => [reqs::create::Ok; reqs::create::Err]);

status_code::direct!(reqs::create::Ok => CREATED);
status_code::map!(reqs::create::Err => [
    AlreadyTaken,
    NoSuchAuthor,
    NoSuchGenre,
    NoSuchTag,
    UnexpectedFileClass,
    InvalidImage,
    FileNotFound,
]);

const _: () = {
    use errors::games::*;
    use status_code::direct;

    direct!(NotFound => NOT_FOUND);
    direct!(NotAnOwner => FORBIDDEN);
    direct!(AlreadyTaken => BAD_REQUEST);
};
