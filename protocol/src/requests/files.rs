pub use Files::get_info;

use crate::requests::Response;
use crate::{errors, types::file};
use eva::service;

#[service(result = Response)]
pub trait Files {
    #[errors(errors::files::NotFound)]
    fn get_info(id: file::Id) -> file::FileInfo;
}
