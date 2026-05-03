use eva::data;

use crate::types::{entity, file};

entity::define_eid! {
    pub struct Id(Upload);
}

#[data]
pub struct Upload {
    pub id: Id,
    pub file_name: Option<file::BaseName>,
    pub uploaded: u64,
    pub size: u64,
    pub class: file::ClassKind,
    pub expected_hash: Option<file::Hash>,
}
