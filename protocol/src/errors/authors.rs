use eva::data;

use crate::types::author::{Selector, Slug};

#[data(error, display("this action will orphan games belonged to the author"))]
pub struct WillOrphanGames;

#[data(error, copy, display("the {author} was not found"))]
pub struct NotFound {
    pub author: Selector,
}

#[data(error, display("@{slug} already exists"))]
pub struct AlreadyExists {
    pub slug: Slug,
}
