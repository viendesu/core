use eva::data;

use crate::{
    errors,
    types::{Patch, True, blog},
};

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub blog: blog::Selector,
    }

    #[data]
    pub struct Ok {
        pub blog: blog::Blog,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::blogs::NotFound),
    }
}

pub mod edit {
    use super::*;

    #[serde_with::apply(Patch => #[serde(default)])]
    #[data]
    pub struct Args {
        pub blog: blog::Selector,
        pub title: Patch<Option<blog::Title>>,
        pub description: Patch<Option<blog::Description>>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::blogs::NotFound),
    }
}
