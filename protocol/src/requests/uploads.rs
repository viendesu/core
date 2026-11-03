use std::num::NonZeroU64;

use crate::requests::Response;
use eva::{True, data, service, str};

use crate::{
    errors,
    types::{file, upload},
    uploads::UploadStream,
};

/// File uploads: `start` reserves one, `finish` streams the bytes.
#[service(result = Response)]
pub trait Uploads {
    /// Reserves an upload against the per-class byte quotas.
    #[flags(write)]
    #[errors(
        errors::uploads::QuotaExceeded,
        errors::uploads::SimUpQuotaExceeded,
        errors::uploads::FileTooBig
    )]
    fn start(
        /// Name of the file. Mainly serves as a hint to user to not
        /// download files with "scary" names. Don't set if that doesn't
        /// matter.
        name: Option<file::BaseName>,
        /// Hash of the file. If specified - verify hashes when finishing.
        hash: Option<file::Hash>,
        /// Class of the file.
        class: this::FileClass,
        /// Size of the file to upload. Must be known
        /// prior to upload, streaming is not supported.
        size: NonZeroU64,
    ) -> Struct! {
        upload: upload::Id,
    };
    embed! {
        #[data(copy)]
        pub enum ImageType {
            #[serde(alias = "image/png")]
            Png,
            #[serde(alias = "image/jpeg")]
            Jpg,
            #[serde(alias = "image/webp")]
            Webp,
            #[serde(alias = "image/bmp")]
            Bmp,
        }

        #[data]
        pub enum FileClass {
            Image(ImageType),
            GameFile,
        }
    }

    /// Streams the file contents of an upload.
    ///
    /// The upload is consumed whatever the outcome; a failed one is
    /// started anew.
    #[flags(write, stream)]
    #[args(data(not(Debug, serde, schemars, Clone, PartialEq)))]
    #[errors(
        errors::uploads::UnableToValidateClass,
        errors::uploads::NotFound,
        errors::uploads::ConcurrentUploadInProgress,
        errors::uploads::HashMismatch,
        errors::uploads::Overuploading,
        errors::uploads::Underuploading,
        errors::uploads::Aborted
    )]
    fn finish(
        id: upload::Id,
        stream: UploadStream<'static>,
    ) -> Struct! {
        file: file::Id,
    };

    /// Cancels an unfinished upload.
    #[flags(write)]
    #[errors(errors::uploads::NotFound)]
    fn abort(upload: upload::Id) -> True;

    /// Lists the caller's unfinished uploads.
    #[error(errors::Impossible)]
    fn list_pending() -> Struct! {
        uploads: Vec<upload::Upload>,
    };
}
