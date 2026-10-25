pub use Articles::{create, delete, edit, get};

use crate::requests::Response;
use eva::int;
use eva::service;

use crate::{
    errors,
    types::{Patch, True, article, blog},
};

pub mod search {
    pub use super::Articles::search::*;
    use super::*;
    #[int(u8, 1..=64)]
    pub enum Limit {}
    impl Default for Limit {
        fn default() -> Self {
            Self::POS24
        }
    }
}

/// Blog articles; content is CommonMark.
#[service(result = Response)]
pub trait Articles {
    /// Lists articles of a blog, newest first, paginated via `before`.
    #[ok(doc = " Results are ordered newest first.")]
    #[errors(errors::blogs::NotFound as NoSuchBlog)]
    fn search(
        blog: blog::Id,
        #[serde(default)] limit: crate::requests::articles::search::Limit,
        before: Option<article::Id>,
    ) -> ok! {
        results: Vec<article::Brief>,
    };
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
    #[errors(errors::blogs::NotFound as NoSuchBlog, errors::blogs::NotAnOwner)]
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
    ) -> True;
    /// Deletes an article; owner only.
    #[flags(write_op)]
    #[errors(errors::articles::NotFound, errors::articles::NotAnOwner)]
    fn delete(article: article::Id) -> True;
}
