use std::num::NonZeroU64;

use eva::data;

use crate::requests::status_code;

use viendesu_core::{
    errors,
    requests::uploads as reqs,
    types::{file, upload},
    uploads::UploadStream,
};

#[data]
pub struct ListPending {}

impl_req!(ListPending => [reqs::list_pending::Ok; reqs::list_pending::Err]);
status_code::direct!(reqs::list_pending::Ok => OK);
status_code::map!(reqs::list_pending::Err => []);

#[data]
pub struct Start {
    pub file_name: Option<file::BaseName>,
    pub hash: Option<file::Hash>,
    pub class: reqs::start::FileClass,
    pub size: NonZeroU64,
}

impl_req!(Start => [reqs::start::Ok; reqs::start::Err]);
status_code::direct!(reqs::start::Ok => CREATED);
status_code::map!(reqs::start::Err => [QuotaExceeded, SimUpQuotaExceeded]);

pub struct Finish {
    pub id: upload::Id,
    pub stream: UploadStream<'static>,
}

impl_req!(Finish => [reqs::finish::Ok; reqs::finish::Err]);
status_code::direct!(reqs::finish::Ok => OK);
status_code::map!(reqs::finish::Err => [
    HashMismatch,
    NotFound,
    Overuploading,
    ConcurrentUploadInProgress,
    UnableToValidateClass,
]);

#[data]
pub struct Abort {}

impl_req!(Abort => [reqs::abort::Ok; reqs::abort::Err]);
status_code::direct!(reqs::abort::Ok => OK);
status_code::map!(reqs::abort::Err => [NotFound]);

const _: () = {
    use errors::uploads::*;
    use status_code::direct;

    direct!(ConcurrentUploadInProgress => BAD_REQUEST);
    direct!(NotFound => NOT_FOUND);
    direct!(QuotaExceeded => TOO_MANY_REQUESTS);
    direct!(SimUpQuotaExceeded => TOO_MANY_REQUESTS);
    direct!(Overuploading => BAD_REQUEST);
    direct!(HashMismatch => BAD_REQUEST);
    direct!(UnableToValidateClass => BAD_REQUEST);
};
