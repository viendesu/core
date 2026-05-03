use std::num::NonZeroU64;

use eva::{data, str};

use crate::{
    errors,
    types::{True, file, upload},
    uploads::UploadStream,
};

pub mod start {
    //! # Upload file to the server.
    //!
    //! The following quotas apply:
    //! - Rate limiting - by bytes, temporal and permanent
    //! - Concurrent uploads limit - also by bytes
    //!
    //! Quotas are applied per-class.

    use super::*;

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

    #[data]
    pub struct Args {
        /// Name of the file. Mainly serves as a hint to user to not
        /// download files with "scary" names. Don't set if that doesn't
        /// matter.
        pub name: Option<file::BaseName>,

        /// Hash of the file. If specified - verify hashes when finishing.
        pub hash: Option<file::Hash>,

        /// Class of the file.
        pub class: FileClass,

        /// Size of the file to upload. Must be known
        /// prior to upload, streaming is not supported.
        pub size: NonZeroU64,
    }

    #[data]
    pub struct Ok {
        pub upload: upload::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        QuotaExceeded(#[from] errors::uploads::QuotaExceeded),
        #[display("{_0}")]
        SimUpQuotaExceeded(#[from] errors::uploads::SimUpQuotaExceeded),
    }
}

pub mod finish {
    use super::*;

    #[data(not(Debug, serde, schemars, Clone, PartialEq))]
    pub struct Args {
        pub id: upload::Id,
        pub stream: UploadStream<'static>,
    }

    #[data]
    pub struct Ok {
        pub file: file::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        UnableToValidateClass(#[from] errors::uploads::UnableToValidateClass),
        #[display("{_0}")]
        NotFound(#[from] errors::uploads::NotFound),
        #[display("{_0}")]
        ConcurrentUploadInProgress(#[from] errors::uploads::ConcurrentUploadInProgress),
        #[display("{_0}")]
        HashMismatch(#[from] errors::uploads::HashMismatch),
        #[display("{_0}")]
        Overuploading(#[from] errors::uploads::Overuploading),
    }
}

pub mod abort {
    use super::*;

    #[data]
    pub struct Args {
        pub upload: upload::Id,
    }

    #[data(copy)]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::uploads::NotFound),
    }
}

pub mod list_pending {
    use super::*;

    #[data]
    pub struct Args {}

    #[data]
    pub struct Ok {
        pub uploads: Vec<upload::Upload>,
    }

    pub type Err = errors::Impossible;
}
