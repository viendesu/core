use crate::requests::Response;
use eva::service;

use crate::{errors, types::file};

#[service(result = Response)]
pub trait Files {
    #[errors(errors::files::NotFound)]
    fn get_info(id: file::Id) -> file::FileInfo;
}
