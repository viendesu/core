use eva::{data, int, str::CompactString};

use crate::{
    errors::{
        self,
        authors::{AlreadyExists, NotFound},
        users as users_errors,
    },
    types::{Patch, True, author, file, user},
};

pub mod search {
    ///! # Search author.
    use super::*;

    #[int(u8, 1..=32)]
    pub enum Limit {}

    impl Default for Limit {
        fn default() -> Self {
            Self::POS15
        }
    }

    #[data]
    pub struct Args {
        pub query: CompactString,
        pub owned_by: Option<user::Selector>,
        pub start_from: Option<author::Id>,
        pub limit: Option<Limit>,
    }

    #[data]
    pub struct Ok {
        pub authors: Vec<author::Author>,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchUser(#[from] errors::users::NotFound),
    }
}

pub mod update {
    ///! # Update author.
    use super::*;

    #[data]
    pub struct Args {
        /// Who to update.
        pub author: author::Selector,
        /// Specific update to apply.
        pub update: Update,
    }

    #[serde_with::apply(Patch => #[serde(default)])]
    #[data]
    pub struct Update {
        pub title: Patch<author::Title>,
        pub description: Patch<Option<author::Description>>,
        pub pfp: Patch<Option<file::Id>>,
        pub slug: Patch<author::Slug>,
        pub verified: Patch<bool>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] NotFound),
    }
}

pub mod get {
    ///! # Get author by selector.
    use super::*;

    #[data]
    pub struct Args {
        pub author: author::Selector,
    }

    #[data]
    pub struct Ok {
        pub author: author::Author,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] NotFound),
    }
}

pub mod create {
    ///! # Create author.
    use super::*;

    #[data]
    pub struct Args {
        pub title: author::Title,
        pub slug: author::Slug,
        pub pfp: Option<file::Id>,
        pub description: Option<author::Description>,
        /// Owner of the author, `None` for current user. Creating
        /// authors for different user than currently authenticated
        /// requires at least `admin` role.
        pub owner: Option<user::Id>,
    }

    #[data]
    pub struct Ok {
        pub id: author::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] NotFound),
        #[display("{_0}")]
        AlreadyExists(#[from] AlreadyExists),
        #[display("{_0}")]
        NoSuchUser(#[from] users_errors::NotFound),
    }
}
