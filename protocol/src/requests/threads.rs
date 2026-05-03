use eva::{data, int, str};

use crate::{
    errors,
    types::{Patch, True, board, message, thread},
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
        #[serde(default)]
        pub limit: Limit,
        // TODO(MKS-6): implement better pagination.
        pub after: Option<thread::Id>,
    }

    #[data]
    pub struct Ok {
        pub results: Vec<thread::Thread>,
    }

    #[data(error, display("_"))]
    pub enum Err {}
}

pub mod delete {
    use super::*;

    #[data]
    pub struct Args {
        pub thread: thread::Id,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::threads::NotFound),
    }
}

pub mod edit {
    use super::*;

    #[data]
    pub struct Args {
        pub thread: thread::Id,
        pub text: Patch<message::Text>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::threads::NotFound),
        #[display("{_0}")]
        NotAnOwner(#[from] errors::threads::NotAnOwner),
    }
}

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub thread: thread::Selector,
    }

    #[data]
    pub struct Ok {
        pub thread: thread::Thread,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::threads::NotFound),
    }
}

pub mod create {
    use super::*;

    #[data]
    pub struct Args {
        pub board: board::Selector,
        pub initial_message: message::Text,
    }

    #[data]
    pub struct Ok {
        pub id: thread::Id,
    }

    #[data]
    pub enum Err {
        #[display("{_0}")]
        NoSuchBoard(#[from] errors::boards::NotFound),
    }
}
