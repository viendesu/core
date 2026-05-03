use eva::data;

use crate::{
    errors,
    types::{Patch, True, board, message},
};

pub mod edit {
    use super::*;

    #[data]
    pub struct Args {
        pub board: board::Selector,
        pub text: Patch<message::Text>,
        pub slug: Patch<Option<board::Slug>>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::boards::NotFound),
    }
}

pub mod delete {
    use super::*;

    #[data]
    pub struct Args {
        pub board: board::Selector,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::boards::NotFound),
    }
}

pub mod create {
    use super::*;

    #[data]
    pub struct Args {
        pub slug: board::Slug,
        pub initial_message: message::Text,
        pub by: Option<message::ById>,
    }

    #[data]
    pub struct Ok {
        pub id: board::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        AlreadyExists(#[from] errors::boards::AlreadyExists),
    }
}

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub board: board::Selector,
    }

    #[data]
    pub struct Ok {
        pub board: board::Board,
    }

    #[data]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::boards::NotFound),
    }
}
