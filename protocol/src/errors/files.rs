use eva::data;

use crate::types::file;

#[data(display("unexpected file class for this type of operation"), error)]
pub struct UnexpectedFileClass {
    pub expected: file::ClassKind,
    pub got: file::ClassKind,
}

#[data(error)]
pub enum InvalidImage {
    #[display("provided image is too big")]
    TooBig,
    #[display("provided image is too small")]
    TooSmall,
}

#[data(copy, display("file {id} was not found"), error)]
pub struct NotFound {
    pub id: file::Id,
}
