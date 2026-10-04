use eva::data;

use crate::types::{author, game};

#[data(error, display("{author} is not an owner of the {game}"))]
pub struct NotAnOwner {
    pub author: author::Id,
    pub game: game::Id,
}

#[data(error, display("the @{slug} is already taken for that author"))]
pub struct AlreadyTaken {
    pub slug: game::Slug,
}

#[data(error, copy, display("the {game} is listed twice or relates to itself"))]
pub struct BadRelation {
    pub game: game::Id,
}

#[data(error, copy, display("the {game} has too many related games"))]
pub struct TooManyRelations {
    pub game: game::Id,
}

#[data(error, copy, display("the {game} was not found"))]
pub struct NotFound {
    pub game: game::Selector,
}
