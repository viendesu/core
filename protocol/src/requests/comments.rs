use std::collections::HashMap;

use crate::requests::Response;
use eva::service;

use crate::{
    errors,
    types::{True, comment, game, user},
};

/// Game comments, threaded one level deep.
#[service(export, result = Response)]
pub trait Comments {
    /// Lists top-level comments of a game.
    ///
    /// Paginated via `before`, plus `before_likes` for `Sort::Top`.
    #[errors(errors::games::NotFound as NoSuchGame)]
    fn list(
        game: game::Id,
        #[serde(default)] sort: comment::Sort,
        /// Exclusive cursor: the last comment of the previous page.
        before: Option<comment::Id>,
        /// Like count of `before` at the time its page was fetched; only
        /// meaningful for [`comment::Sort::Top`]. When omitted, the
        /// comment's current like count is used instead.
        before_likes: Option<comment::Likes>,
        #[serde(default)] limit: comment::Limit,
    ) -> ok! {
        #![doc = " Top-level comments only; fetch replies via `replies`."]

        comments: Vec<comment::Comment>,
        users: HashMap<user::Id, user::Mini>,
    };

    /// Lists replies to a top-level comment, paginated via `after`.
    #[errors(errors::comments::NotFound)]
    fn replies(
        comment: comment::Id,
        /// Exclusive cursor: the last reply of the previous page.
        after: Option<comment::Id>,
        #[serde(default)] limit: comment::Limit,
    ) -> ok! {
        #![doc = " Results are ordered oldest first."]

        comments: Vec<comment::Comment>,
        users: HashMap<user::Id, user::Mini>,
    };

    /// Comments on a game, or replies to a top-level comment via `parent`.
    #[flags(write_op)]
    #[errors(
        errors::games::NotFound as NoSuchGame,
        errors::comments::NotFound as NoSuchParent,
        errors::comments::InvalidParent,
    )]
    fn create(
        game: game::Id,
        text: comment::Text,
        /// Reply to this top-level comment; threads are one level deep.
        #[serde(default)]
        parent: Option<comment::Id>,
    ) -> ok! {
        id: comment::Id,
    };

    /// Replaces a comment's text; owner only.
    #[flags(write_op)]
    #[errors(errors::comments::NotFound)]
    fn edit(comment: comment::Id, text: comment::Text) -> ok!(True);

    /// Deletes a comment; owner only.
    ///
    /// A top-level comment with replies is tombstoned instead: its text
    /// and author are wiped, the replies stay.
    #[flags(write_op)]
    #[errors(errors::comments::NotFound)]
    fn delete(comment: comment::Id) -> ok!(True);

    /// Likes a comment or retracts the like.
    #[flags(write_op)]
    #[errors(errors::comments::NotFound)]
    fn like(
        comment: comment::Id,
        /// `false` retracts the caller's like.
        liked: bool,
    ) -> ok! {
        /// Like count after the operation.
        likes: comment::Likes,
    };
}
