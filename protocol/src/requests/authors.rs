use std::collections::HashMap;

use crate::requests::Response;
use eva::{True, data, int, service, str::CompactString};

use crate::{
    errors,
    types::{Patch, author, file, user},
};

/// Game authors: developers and circles.
#[service(result = Response)]
pub trait Authors {
    /// Searches authors by text query, optionally owned by a user.
    ///
    /// Paginated via `start_from`.
    #[errors(NoSuchUser(errors::users::NotFound))]
    fn search(
        query: CompactString,
        owned_by: Option<user::Selector>,
        start_from: Option<author::Id>,
        limit: Option<this![Limit]>,
    ) -> ok! {
        authors: Vec<author::Author>,
        users: HashMap<user::Id, user::Mini>,
    };
    embed! {
        #[int(u8, 1..=32)]
        pub enum Limit {}

        impl Default for Limit {
            fn default() -> Self {
                Self::POS15
            }
        }
    }

    /// Patches an author.
    #[flags(write_op)]
    #[errors(
        errors::authors::NotFound,
        errors::authors::AlreadyExists,
        FileNotFound(errors::files::NotFound),
        errors::files::UnexpectedFileClass
    )]
    fn update(
        /// Who to update.
        author: author::Selector,
        /// Specific update to apply.
        update: this![Update],
    ) -> True;
    embed! {
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

    /// Gets an author.
    #[errors(errors::authors::NotFound)]
    fn get(
        author: author::Selector,
    ) -> ok! {
        author: author::Author,
        users: HashMap<user::Id, user::Mini>,
    };

    /// Creates an author.
    #[flags(write_op)]
    #[errors(
        errors::authors::NotFound,
        errors::authors::AlreadyExists,
        NoSuchUser(errors::users::NotFound),
        FileNotFound(errors::files::NotFound),
        errors::files::UnexpectedFileClass
    )]
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
