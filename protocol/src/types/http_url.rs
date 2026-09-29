use std::{borrow::Cow, str::FromStr};

use eva::{data, url::Url};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};

#[data(copy, error, display(doc))]
pub enum ParseError {
    /// Malformed URL.
    Malformed,
    /// Only http and https URLs are allowed.
    Scheme,
}

/// Absolute `http` or `https` URL: other schemes (`javascript:` and the
/// like) are unsafe to render as a link, so they never deserialize.
#[data(not(serde, schemars), display("{_0}"))]
pub struct HttpUrl(Url);

impl HttpUrl {
    pub fn new(url: Url) -> Result<Self, ParseError> {
        match url.scheme() {
            "http" | "https" => Ok(Self(url)),
            _ => Err(ParseError::Scheme),
        }
    }

    pub const fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn into_url(self) -> Url {
        self.0
    }
}

impl TryFrom<Url> for HttpUrl {
    type Error = ParseError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        Self::new(url)
    }
}

impl FromStr for HttpUrl {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.parse().map_err(|_| ParseError::Malformed)?)
    }
}

impl<'de> Deserialize<'de> for HttpUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(Url::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

impl Serialize for HttpUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl JsonSchema for HttpUrl {
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::HttpUrl"))
    }

    fn schema_name() -> Cow<'static, str> {
        "HttpUrl".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "format": "uri",
            "pattern": "^[Hh][Tt][Tt][Pp][Ss]?://",
        })
    }
}
