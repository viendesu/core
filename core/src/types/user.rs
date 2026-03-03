use std::{borrow::Cow, num::NonZeroI64};

use serde::{Serialize, de};

use eva::{
    data, rand, str,
    str::{CompactString, HasPattern, ParseError, Seq, ascii},
    zst_error,
};

use crate::types::{entity, file, slug::Slug};

entity::define_eid! {
    pub struct AuthSessionId(AuthSession);
}

#[str(newtype, copy)]
pub struct TelegramUsername(Slug<23>);

#[data(copy, ord, display("tg:{_0}"))]
pub struct TelegramId(pub NonZeroI64);

#[data]
pub struct TelegramAuth {
    pub bot_username: TelegramUsername,
}

#[data]
pub enum AuthoredAuth {
    Telegram(#[from] TelegramAuth),
}

#[data]
pub enum DirectAuth {
    Password,
}

#[data]
pub enum AuthMethod {
    Authored(#[from] AuthoredAuth),
    Direct(DirectAuth),
}

/// Miniature for the user.
#[data]
pub struct Mini {
    pub id: Id,
    pub nickname: Nickname,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pfp: Option<file::Id>,
}

#[data]
pub struct User {
    pub id: Id,
    pub nickname: Nickname,
    pub display_name: Option<DisplayName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<Email>,
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<Bio>,
    pub pfp: Option<file::Id>,
}

#[data(copy)]
#[serde(untagged)]
pub enum Selector {
    #[serde(with = "NicknameStr")]
    #[display("@{_0}")]
    Nickname(#[from] Nickname),
    #[display("{_0}")]
    Id(#[from] Id),
}

struct NicknameStr;

impl schemars::JsonSchema for NicknameStr {
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::NicknameStr"))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("NicknameStr")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut pat = String::from("^@");
        Nickname::pat_into(&mut pat);
        pat.push('$');
        schemars::json_schema!({
            "type": "string",
            "pattern": pat,
        })
    }
}

impl NicknameStr {
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(des: D) -> Result<Nickname, D::Error> {
        let s = <&'de str as serde::Deserialize<'de>>::deserialize(des)?;
        if let Some(nickname) = s.strip_prefix('@') {
            nickname.parse().map_err(de::Error::custom)
        } else {
            Err(de::Error::custom(zst_error!(
                "nickname must start with a '@'"
            )))
        }
    }

    pub fn serialize<S: serde::Serializer>(nick: &Nickname, ser: S) -> Result<S::Ok, S::Error> {
        eva::str::format_compact!("@{nick}").serialize(ser)
    }
}

#[str(fixed(error = ParseError))]
pub struct SignUpCompletionToken(Seq<16, ascii::UrlSafe>);

// TODO: impl this from macro.
impl rand::distr::Distribution<SignUpCompletionToken> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> SignUpCompletionToken {
        let seq = std::array::from_fn(|_| rng.random());
        SignUpCompletionToken(Seq(seq))
    }
}

/// Email of the user.
#[str(newtype)]
pub struct Email(pub CompactString);

/// Hashed user password.
#[str(newtype)]
pub struct PasswordHash(pub CompactString);

/// Cleartext user password.
#[str(newtype)]
pub struct Password(pub CompactString);

entity::define_eid! {
    /// ID of the user.
    pub struct Id(User);
}

/// Machine and user-readable identifier.
#[str(newtype, copy)]
pub struct Nickname(pub Slug<23>);

/// Preferred name for display.
#[str(newtype)]
pub struct DisplayName(pub CompactString);

/// User's bio.
#[str(newtype)]
pub struct Bio(pub CompactString);

#[data(copy, ord, display(name))]
#[derive(Default)]
pub enum Role {
    /// The least powerful.
    Slave,
    /// Plain user.
    #[default]
    User,
    /// Moderator.
    Mod,
    /// Admin of the site.
    Admin,
    /// The most powerful user.
    Master,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple role ordering test.
    #[test]
    fn role_ordering() {
        let ordering = [Role::Admin, Role::Mod, Role::User, Role::Slave];
        let mut prev_role = Role::Master;
        for role in ordering {
            assert!(
                prev_role > role,
                "{prev_role} <= {role}, must be {prev_role} > {role}",
            );
            prev_role = role;
        }
    }
}
