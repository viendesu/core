use eva::{data, int};

use crate::{
    errors,
    types::{Patch, True, article, blog},
};

pub mod search {
    use super::*;

    #[int(u8, 1..=64)]
    pub enum Limit {}

    impl Default for Limit {
        fn default() -> Self {
            Self::POS24
        }
    }

    #[data]
    pub struct Args {
        pub blog: blog::Id,
        #[serde(default)]
        pub limit: Limit,
        // TODO(MKS-6): implement better pagination.
        pub before: Option<article::Id>,
    }

    /// Results are ordered newest first.
    #[data]
    pub struct Ok {
        pub results: Vec<article::Brief>,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchBlog(#[from] errors::blogs::NotFound),
    }
}

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub article: article::Selector,
    }

    #[data]
    pub struct Ok {
        pub article: article::Article,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::articles::NotFound),
    }
}

pub mod create {
    use super::*;

    #[data]
    pub struct Args {
        pub blog: blog::Id,
        pub title: article::Title,
        pub content: article::Content,
    }

    #[data]
    pub struct Ok {
        pub id: article::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchBlog(#[from] errors::blogs::NotFound),
        #[display("{_0}")]
        NotAnOwner(#[from] errors::blogs::NotAnOwner),
    }
}

pub mod edit {
    use super::*;

    #[serde_with::apply(Patch => #[serde(default)])]
    #[data]
    pub struct Args {
        pub article: article::Id,
        pub title: Patch<article::Title>,
        pub content: Patch<article::Content>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::articles::NotFound),
        #[display("{_0}")]
        NotAnOwner(#[from] errors::articles::NotAnOwner),
    }
}

pub mod delete {
    use super::*;

    #[data]
    pub struct Args {
        pub article: article::Id,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::articles::NotFound),
        #[display("{_0}")]
        NotAnOwner(#[from] errors::articles::NotAnOwner),
    }
}
