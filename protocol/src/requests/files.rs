use eva::data;

use crate::{errors, types::file};

pub mod get_info {
    use super::*;

    #[data]
    pub struct Args {
        pub id: file::Id,
    }

    #[data]
    pub struct Ok {
        pub info: file::FileInfo,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::files::NotFound),
    }
}
