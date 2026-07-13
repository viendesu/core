//! Crate-internal deserialization helpers.

use std::{borrow::Cow, fmt};

use serde::de;

struct CowStrVisitor;

impl<'de> de::Visitor<'de> for CowStrVisitor {
    type Value = Cow<'de, str>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a string")
    }

    fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
        Ok(Cow::Borrowed(v))
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        Ok(Cow::Owned(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        Ok(Cow::Owned(v))
    }
}

/// Deserializes a string, borrowing from the input when possible and copying
/// otherwise. Unlike `&str` this accepts transient strings, which non-slice
/// deserializers (e.g. `serde_json::from_value`) hand out.
pub(crate) fn cow_str<'de, D>(deserializer: D) -> Result<Cow<'de, str>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(CowStrVisitor)
}
