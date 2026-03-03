use eva::{data, error::ShitHappens};
use eyre::Context;

#[data(copy, display("got unsupported mime type"), error)]
pub struct UnknownMimeType;

#[data(copy, display("{}", self.mime_type()))]
#[derive(Default)]
pub enum Format {
    #[default]
    Json,
}

impl Format {
    pub fn from_mime_type(mime: &str) -> Result<Self, UnknownMimeType> {
        use Format::*;
        use UnknownMimeType as E;

        match mime {
            "application/json" => Ok(Json),
            "*/*" => Ok(Json),
            _ => Err(E),
        }
    }

    pub const fn mime_type(self) -> &'static str {
        use Format::*;

        match self {
            Json => "application/json",
        }
    }
}

#[data]
#[derive(Default)]
pub struct DumpParams {
    pub pretty: bool,
}

impl Format {
    pub fn dump<T: ?Sized>(self, params: DumpParams, what: &T, dst: &mut Vec<u8>)
    where
        T: serde::Serialize,
    {
        match self {
            Self::Json => if params.pretty {
                serde_json::to_writer_pretty(dst, what)
            } else {
                serde_json::to_writer(dst, what)
            }
            .shit_happens(),
        }
    }

    pub fn load<'de, T>(&self, buf: &'de [u8]) -> eyre::Result<T>
    where
        T: serde::Deserialize<'de>,
    {
        match self {
            Self::Json => serde_json::from_slice(buf).wrap_err("failed to deserialize JSON"),
        }
    }
}
