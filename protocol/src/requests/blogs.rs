use eva::data;

use crate::{
    errors,
    types::{Patch, True, blog},
};

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub blog: blog::Id,
    }

    #[data]
    pub struct Ok {
        pub blog: blog::Blog,
        pub owner: blog::Owner,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::blogs::NotFound),
    }
}

// `edit` with every field patched to `None` deletes the blog metadata
// entirely: a blog with empty metadata is not stored.
pub mod edit {
    use super::*;

    #[serde_with::apply(Patch => #[serde(default)])]
    #[data]
    pub struct Args {
        pub blog: blog::Id,
        pub title: Patch<Option<blog::Title>>,
        pub description: Patch<Option<blog::Description>>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::blogs::NotFound),
        #[display("{_0}")]
        NotAnOwner(#[from] errors::blogs::NotAnOwner),
    }
}
