use eva::data;

use crate::{
    errors,
    types::{True, message, thread},
};

pub mod edit {
    use super::*;

    #[data]
    pub struct Args {
        pub message: message::Id,
        pub text: message::Text,
    }

    #[data]
    pub struct Ok;

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::messages::NotFound),
    }
}

pub mod delete {
    use super::*;

    #[data]
    pub struct Args {
        pub message: message::Id,
    }

    #[data]
    pub struct Ok(pub True);

    #[data]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::messages::NotFound),
    }
}

pub mod post {
    use super::*;

    #[data]
    pub struct Args {
        pub thread: thread::Selector,
        pub text: message::Text,
    }

    #[data]
    pub struct Ok {
        pub id: message::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchThread(#[from] errors::threads::NotFound),
    }
}

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub message: message::Selector,
    }

    #[data]
    pub struct Ok {
        pub message: message::Message,
    }

    #[data]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::messages::NotFound),
    }
}
