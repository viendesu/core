use eva::data;

use crate::requests::status_code;

use viendesu_core::{
    errors,
    requests::games as reqs,
    types::{Patch, author, file, game, mark},
};

#[data]
#[serde_with::apply(Patch => #[serde(default)])]
pub struct Update {
    pub title: Patch<game::Title>,
    pub description: Patch<Option<game::Description>>,
    pub slug: Patch<game::Slug>,
    pub thumbnail: Patch<Option<file::Id>>,
    pub downloads: Patch<Vec<game::Download>>,
    pub genres: Patch<mark::Genres>,
    pub badges: Patch<mark::Badges>,
    pub tags: Patch<mark::Tags>,
    pub screenshots: Patch<game::Screenshots>,
    pub published: Patch<bool>,
}

impl_req!(Update => [reqs::update::Ok; reqs::update::Err]);

status_code::direct!(reqs::update::Ok => OK);
status_code::map!(reqs::update::Err => [
    NotFound,
    NoSuchBadge,
    NoSuchGenre,
    NoSuchTag,
    UnexpectedFileClass,
    InvalidImage,
]);

#[data]
pub struct Search {
    pub query: Option<game::SearchQuery>,
    pub author: Option<author::Selector>,
    #[serde(default)]
    pub include: reqs::search::Marks,
    #[serde(default)]
    pub exclude: reqs::search::Marks,
    #[serde(default)]
    pub order: reqs::search::Order,
    #[serde(default)]
    pub sort_by: reqs::search::SortBy,
    pub limit: Option<reqs::search::Limit>,
}

impl_req!(Search => [reqs::search::Ok; reqs::search::Err]);

status_code::direct!(reqs::search::Ok => OK);
status_code::map!(reqs::search::Err => [NoSuchAuthor]);

#[data]
pub struct Get {
    #[serde(default)]
    pub resolve_marks: bool,
}

impl_req!(Get => [reqs::get::Ok; reqs::get::Err]);

status_code::direct!(reqs::get::Ok => OK);
status_code::map!(reqs::get::Err => [NotFound, NoSuchAuthor]);

#[data]
pub struct Create {
    pub title: game::Title,
    pub description: Option<game::Description>,
    pub thumbnail: Option<file::Id>,
    pub author: author::Id,
    #[serde(default)]
    pub downloads: Vec<game::Download>,
    pub slug: Option<game::Slug>,
    #[serde(default)]
    pub screenshots: game::Screenshots,
    #[serde(default)]
    pub tags: mark::Tags,
    #[serde(default)]
    pub genres: mark::Genres,
    pub vndb: Option<game::VndbId>,
    pub release_date: Option<game::ReleaseDate>,
}

impl_req!(Create => [reqs::create::Ok; reqs::create::Err]);

status_code::direct!(reqs::create::Ok => CREATED);
status_code::map!(reqs::create::Err => [
    AlreadyTaken,
    NoSuchAuthor,
    NoSuchGenre,
    NoSuchTag,
    UnexpectedFileClass,
    InvalidImage,
]);

const _: () = {
    use errors::games::*;
    use status_code::direct;

    direct!(NotFound => NOT_FOUND);
    direct!(NotAnOwner => FORBIDDEN);
    direct!(AlreadyTaken => BAD_REQUEST);
};
