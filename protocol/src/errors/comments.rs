use eva::data;

use crate::types::comment;

#[data(error, copy, display("the {comment} was not found"))]
pub struct NotFound {
    pub comment: comment::Id,
}

/// The parent is itself a reply (threads are one level deep) or belongs
/// to another game.
#[data(error, copy, display("the {parent} cannot be replied to"))]
pub struct InvalidParent {
    pub parent: comment::Id,
}
