use crate::requests::Response;
use eva::service;

use crate::{
    errors,
    types::{Patch, True, board, message},
};

/// Forum boards.
#[service(result = Response)]
pub trait Boards {
    /// Patches a board.
    #[flags(write_op)]
    #[errors(errors::boards::NotFound)]
    fn edit(
        board: board::Selector,
        text: Patch<message::Text>,
        slug: Patch<Option<board::Slug>>,
    ) -> True;

    /// Deletes a board.
    #[flags(write_op)]
    #[errors(errors::boards::NotFound)]
    fn delete(board: board::Selector) -> True;

    /// Creates a board with its initial message.
    #[flags(write_op)]
    #[errors(errors::boards::AlreadyExists)]
    fn create(
        slug: board::Slug,
        initial_message: message::Text,
        by: Option<message::ById>,
    ) -> ok! {
        id: board::Id,
    };

    /// Gets a board.
    #[errors(errors::boards::NotFound)]
    fn get(
        board: board::Selector,
    ) -> ok! {
        board: board::Board,
    };
}
