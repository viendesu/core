use crate::requests::Response;
use eva::service;

use crate::{
    errors,
    types::{Patch, True, blog},
};

// `edit` with every field patched to `None` deletes the blog metadata
// entirely: a blog with empty metadata is not stored.

/// Blogs: a facet of a user, author or game, keyed by its id.
#[service(export, result = Response)]
pub trait Blogs {
    /// Gets a blog with its resolved owner.
    #[errors(
        errors::blogs::NotFound,
        NoSuchUser(errors::users::NotFound),
        NoSuchAuthor(errors::authors::NotFound),
        NoSuchGame(errors::games::NotFound)
    )]
    fn get(
        blog: blog::Selector,
    ) -> ok! {
        blog: blog::Blog,
        owner: blog::Owner,
    };

    /// Patches blog metadata; owner only.
    ///
    /// Patching every field to `None` deletes the metadata.
    #[flags(write_op)]
    #[args(serde_with::apply(Patch => #[serde(default)]))]
    #[errors(errors::blogs::NotFound, errors::blogs::NotAnOwner)]
    fn edit(
        blog: blog::Id,
        title: Patch<Option<blog::Title>>,
        description: Patch<Option<blog::Description>>,
    ) -> ok!(True);
}
