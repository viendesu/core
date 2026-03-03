use eva::{bytesize::ByteSize, data};

use crate::types::{file, upload};

#[data(error, display("unable to validate file class: {error}"))]
pub struct UnableToValidateClass {
    pub error: String,
}

#[data(error, copy, display("someone is finishing file already"))]
#[derive(Default)]
pub enum ConcurrentUploadInProgress {
    #[serde(rename = "concurrent_upload_in_progress")]
    #[default]
    Value,
}

#[data(error, display("hash mismatch, expected: {expected}, got: {got}"))]
pub struct HashMismatch {
    pub expected: file::Hash,
    pub got: file::Hash,
}

#[data(
    error,
    copy,
    display("uploading more than requested ({expected} bytes)")
)]
pub struct Overuploading {
    pub expected: u64,
}

#[data(error, copy, display("simultaneous upload quota exceeded: {}/{}", ByteSize::b(*in_progress), ByteSize::b(*quota)))]
pub struct SimUpQuotaExceeded {
    pub in_progress: u64,
    pub quota: u64,
}

#[data(
    error,
    copy,
    display(
        "file uploading quota exceeded({}/{})",
        ByteSize::b(*uploaded),
        ByteSize::b(*quota)
    )
)]
pub struct QuotaExceeded {
    pub uploaded: u64,
    pub quota: u64,
}

#[data(error, copy, display("{id} was not found"))]
pub struct NotFound {
    pub id: upload::Id,
}
