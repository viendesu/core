use eva::{data, trait_set};

use axum::http::request::Parts;

use viendesu_core::errors::AuxResult;

use crate::{format::Format, requests::Request};

pub mod extract;

#[data]
pub struct MetaInfo {
    pub request_format: Format,
}

impl MetaInfo {
    pub fn gather(parts: &Parts) -> AuxResult<Self> {
        let request_format = extract::request_format(parts)?;

        Ok(Self { request_format })
    }
}

trait_set! {
    pub trait ServerRequest = Request;
}
