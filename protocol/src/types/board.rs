use crate::types::{entity, game, slug, user};

use std::borrow::Cow;

use eva::{
    data, str,
    str::{HasPattern, format_compact},
    time::Timestamp,
    zst_error,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};

#[data(copy)]
#[serde(untagged)]
pub enum Selector {
    #[display("@{_0}")]
    #[serde(with = "Slugged")]
    Slug(#[from] Slug),
    #[display("{_0}")]
    Id(#[from] Id),
}

struct Slugged;

impl JsonSchema for Slugged {
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::Slugged"))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Slugged")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut pat = String::from("^@");
        Slug::pat_into(&mut pat);
        pat.push('$');
        schemars::json_schema!({
            "type": "string",
            "pattern": pat,
            "description": "slug of the board"
        })
    }
}

impl Slugged {
    fn serialize<S>(slug: &Slug, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format_compact!("@{slug}").serialize(serializer)
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Slug, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&'de str as Deserialize<'de>>::deserialize(deserializer)?;
        if let Some(rest) = s.strip_prefix('@') {
            let slug: Slug = rest.parse().map_err(de::Error::custom)?;
            Ok(slug)
        } else {
            Err(de::Error::custom(zst_error!("slug must start with a @")))
        }
    }
}

entity::define_eid! {
    pub struct Id(Board);
}

entity::define_eid! {
    pub struct Op(User | Game);
}

impl Op {
    pub const fn user(user: user::Id) -> Self {
        Self(user.raw_id())
    }

    pub const fn game(game: game::Id) -> Self {
        Self(game.raw_id())
    }
}

#[str(newtype, copy)]
pub struct Slug(slug::LowerSlug<7>);

#[str(newtype)]
pub struct Title(str::CompactString);

#[data]
pub struct Brief {
    pub id: Id,
    pub slug: Option<Slug>,
}

#[data]
pub struct Board {
    pub id: Id,
    pub title: Option<Title>,
    pub slug: Option<Slug>,
    pub op: Op,
    pub created_at: Timestamp,
}
