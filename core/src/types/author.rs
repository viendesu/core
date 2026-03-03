use std::borrow::Cow;

use eva::{data, str, str::CompactString, time::Date, zst_error};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};

use crate::types::{entity, file, slug, user};

/// Author selector.
#[data(copy, ord)]
#[serde(untagged)]
pub enum Selector {
    #[display("{}", _0.to_str())]
    Id(#[from] Id),
    #[display("@{_0}")]
    #[serde(with = "SlugStr")]
    Slug(#[from] Slug),
}

struct SlugStr;

impl JsonSchema for SlugStr {
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::SlugStr"))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("SlugStr")
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut pat = String::from("^@");
        <Slug as str::HasPattern>::pat_into(&mut pat);
        pat.push('$');

        schemars::json_schema!({
            "type": "string",
            "pattern": pat,
        })
    }
}

impl SlugStr {
    fn serialize<S: serde::Serializer>(slug: &Slug, serializer: S) -> Result<S::Ok, S::Error> {
        str::format_compact!("@{slug}").serialize(serializer)
    }

    fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Slug, D::Error> {
        let s = <&'de str as Deserialize<'de>>::deserialize(deserializer)?;
        if let Some(slug) = s.strip_prefix('@') {
            slug.parse().map_err(de::Error::custom)
        } else {
            Err(de::Error::custom(zst_error!("slug must start with a @")))
        }
    }
}

/// Author miniature.
#[data]
pub struct Mini {
    pub id: Id,
    pub title: Title,
    pub slug: Slug,
    pub owner: user::Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp: Option<file::Id>,
}

/// Author.
#[data]
pub struct Author {
    pub id: Id,
    pub slug: Slug,
    pub title: Title,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Description>,
    pub owner: user::Mini,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp: Option<file::Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<Verification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<ContactEmail>,
    pub created_at: Date,
}

/// Information about author verification.
#[data]
pub struct Verification {
    pub by: user::Mini,
    pub at: Date,
}

/// Email for contacting the author.
#[data]
pub struct ContactEmail(pub CompactString);

entity::define_eid! {
    /// ID of the author.
    pub struct Id(Author);
}

/// Author's slug.
#[str(newtype, copy)]
pub struct Slug(pub slug::Slug<23>);

/// Author's title.
#[str(newtype)]
pub struct Title(pub CompactString);

/// Author's description.
#[str(newtype)]
pub struct Description(pub CompactString);
