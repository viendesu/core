use eva::data;

use crate::types::{entity, tab};

#[data(error, display("no such tab: {id}"))]
pub struct NoSuchTab {
    pub id: tab::Id,
}

#[data(error, display("the item {item} already exists in {id}"))]
pub struct Duplicate {
    pub id: tab::Id,
    pub item: entity::Id,
}

#[data(error, display("the item {item} was not found in {id}"))]
pub struct NoSuchItem {
    pub id: tab::Id,
    pub item: entity::Id,
}

#[data(error, display("invalid tab kind, expected {expected}, but got {got}"))]
pub struct InvalidKind {
    pub expected: tab::Kind,
    pub got: tab::Kind,
}

#[data(
    copy,
    error,
    display("invalid ID, expected game or author, got: {got}")
)]
pub struct InvalidItemId {
    pub got: entity::Kind,
}
