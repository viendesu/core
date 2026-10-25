use crate::requests::Response;
use eva::int;
use eva::service;

use crate::{
    errors,
    types::{Patch, True, article, blog},
};

/// Blog articles; content is CommonMark.
#[service(export, result = Response)]
pub trait Articles {
    /// Lists articles of a blog, newest first, paginated via `before`.
    #[errors(NoSuchBlog(errors::blogs::NotFound))]
    fn search(
        blog: blog::Id,
        #[serde(default)] limit: this![Limit],
        before: Option<article::Id>,
    ) -> ok! {
        #![doc = " Results are ordered newest first."]

        results: Vec<article::Brief>,
    };
    embed! {
        #[int(u8, 1..=64)]
        pub enum Limit {}

        impl Default for Limit {
            fn default() -> Self {
                Self::POS24
            }
        }
    }

    /// Gets an article.
    #[errors(errors::articles::NotFound)]
    fn get(
        article: article::Selector,
        /// Only match the article if it belongs to this blog. Everything
        /// else is `NotFound`.
        #[serde(default)]
        blog: Option<blog::Selector>,
    ) -> ok! {
        article: article::Article,
    };

    /// Publishes an article; requires owning the blog.
    #[flags(write_op)]
    #[errors(NoSuchBlog(errors::blogs::NotFound), errors::blogs::NotAnOwner)]
    fn create(
        blog: blog::Id,
        title: article::Title,
        content: article::Content,
    ) -> ok! {
        id: article::Id,
    };

    /// Patches an article; owner only.
    #[flags(write_op)]
    #[args(serde_with::apply(Patch => #[serde(default)]))]
    #[errors(errors::articles::NotFound, errors::articles::NotAnOwner)]
    fn edit(
        article: article::Id,
        title: Patch<article::Title>,
        content: Patch<article::Content>,
    ) -> ok!(True);

    /// Deletes an article; owner only.
    #[flags(write_op)]
    #[errors(errors::articles::NotFound, errors::articles::NotAnOwner)]
    fn delete(article: article::Id) -> ok!(True);
}
