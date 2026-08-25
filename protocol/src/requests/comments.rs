use std::collections::HashMap;

use crate::{
    errors,
    types::{True, comment, game, user},
};

use eva::data;

pub mod list {
    use super::*;

    #[data]
    pub struct Args {
        pub game: game::Id,
        #[serde(default)]
        pub sort: comment::Sort,
        /// Exclusive cursor: the last comment of the previous page.
        pub before: Option<comment::Id>,
        /// Like count of `before` at the time its page was fetched; only
        /// meaningful for [`comment::Sort::Top`]. When omitted, the
        /// comment's current like count is used instead.
        pub before_likes: Option<comment::Likes>,
        #[serde(default)]
        pub limit: comment::Limit,
    }

    /// Top-level comments only; fetch replies via `replies`.
    #[data]
    pub struct Ok {
        pub comments: Vec<comment::Comment>,
        pub users: HashMap<user::Id, user::Mini>,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchGame(#[from] errors::games::NotFound),
    }
}

pub mod replies {
    use super::*;

    #[data]
    pub struct Args {
        pub comment: comment::Id,
        /// Exclusive cursor: the last reply of the previous page.
        pub after: Option<comment::Id>,
        #[serde(default)]
        pub limit: comment::Limit,
    }

    /// Results are ordered oldest first.
    #[data]
    pub struct Ok {
        pub comments: Vec<comment::Comment>,
        pub users: HashMap<user::Id, user::Mini>,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::comments::NotFound),
    }
}

pub mod create {
    use super::*;

    #[data]
    pub struct Args {
        pub game: game::Id,
        pub text: comment::Text,
        /// Reply to this top-level comment; threads are one level deep.
        #[serde(default)]
        pub parent: Option<comment::Id>,
    }

    #[data]
    pub struct Ok {
        pub id: comment::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchGame(#[from] errors::games::NotFound),
        #[display("{_0}")]
        NoSuchParent(#[from] errors::comments::NotFound),
        #[display("{_0}")]
        InvalidParent(#[from] errors::comments::InvalidParent),
    }
}

pub mod edit {
    use super::*;

    #[data]
    pub struct Args {
        pub comment: comment::Id,
        pub text: comment::Text,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::comments::NotFound),
    }
}

pub mod delete {
    use super::*;

    #[data]
    pub struct Args {
        pub comment: comment::Id,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::comments::NotFound),
    }
}

pub mod like {
    use super::*;

    #[data]
    pub struct Args {
        pub comment: comment::Id,
        /// `false` retracts the caller's like.
        pub liked: bool,
    }

    #[data]
    pub struct Ok {
        /// Like count after the operation.
        pub likes: comment::Likes,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::comments::NotFound),
    }
}
