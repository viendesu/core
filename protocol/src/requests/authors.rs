use crate::requests::Response;
use std::collections::HashMap;

pub use Authors::{create, get};

use eva::{data, int, service, str::CompactString};

use crate::{
    errors::{
        self,
        authors::{AlreadyExists, NotFound},
        users as users_errors,
    },
    types::{Patch, True, author, file, user},
};

pub mod search {
    pub use super::Authors::search::*;
    use super::*;
    #[int(u8, 1..=32)]
    pub enum Limit {}
    impl Default for Limit {
        fn default() -> Self {
            Self::POS15
        }
    }
}

pub mod update {
    pub use super::Authors::update::*;
    use super::*;
    #[serde_with::apply(Patch => #[serde(default)])]
    #[data]
    pub struct Update {
        pub title: Patch<author::Title>,
        pub description: Patch<Option<author::Description>>,
        pub pfp: Patch<Option<file::Id>>,
        pub slug: Patch<author::Slug>,
        pub verified: Patch<bool>,
    }
}

/// Game authors: developers and circles.
#[service(result = Response)]
pub trait Authors {
    /// Searches authors by text query, optionally owned by a user.
    ///
    /// Paginated via `start_from`.
    #[errors(errors::users::NotFound as NoSuchUser)]
    fn search(
        query: CompactString,
        owned_by: Option<user::Selector>,
        start_from: Option<author::Id>,
        limit: Option<crate::requests::authors::search::Limit>,
    ) -> ok! {
        authors: Vec<author::Author>,
        users: HashMap<user::Id, user::Mini>,
    };
    /// Patches an author.
    #[flags(write_op)]
    #[errors(NotFound, AlreadyExists, errors::files::NotFound as FileNotFound, errors::files::UnexpectedFileClass)]
    fn update(
        /// Who to update.
        author: author::Selector,
        /// Specific update to apply.
        update: crate::requests::authors::update::Update,
    ) -> True;
    /// Gets an author.
    #[errors(NotFound)]
    fn get(
        author: author::Selector,
    ) -> ok! {
        author: author::Author,
        users: HashMap<user::Id, user::Mini>,
    };
    /// Creates an author.
    #[flags(write_op)]
    #[errors(NotFound, AlreadyExists, users_errors::NotFound as NoSuchUser, errors::files::NotFound as FileNotFound, errors::files::UnexpectedFileClass)]
    fn create(
        title: author::Title,
        slug: author::Slug,
        pfp: Option<file::Id>,
        description: Option<author::Description>,
        /// Owner of the author, `None` for current user. Creating
        /// authors for different user than currently authenticated
        /// requires at least `admin` role.
        owner: Option<user::Id>,
    ) -> ok! {
        id: author::Id,
    };
}
