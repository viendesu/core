use std::borrow::Cow;

use eva::data;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};

#[data(copy, ord, error, display("must be `true`"))]
pub struct MustBeTrue;

#[data(copy, ord, not(serde, schemars), display(doc))]
#[derive(Default)]
pub enum True {
    /// true.
    #[default]
    Value,
}

impl JsonSchema for True {
    fn schema_id() -> Cow<'static, str> {
        bool::schema_id()
    }

    fn schema_name() -> Cow<'static, str> {
        bool::schema_name()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({"const": true})
    }
}

impl True {
    pub const fn new() -> Self {
        Self::Value
    }
}

impl<'de> Deserialize<'de> for True {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let res = bool::deserialize(deserializer)?;
        if res {
            Ok(True::Value)
        } else {
            Err(de::Error::custom(MustBeTrue))
        }
    }
}

impl Serialize for True {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        true.serialize(serializer)
    }
}
