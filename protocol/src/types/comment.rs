use eva::{data, int, str, time::Timestamp};

use crate::types::{entity, user};

entity::define_eid! {
    /// ID of the comment.
    pub struct Id(Comment);
}

/// Text of the comment.
#[str(newtype)]
pub struct Text(str::CompactString);

/// Number of likes from users.
#[data(copy, ord, display("{_0}"))]
#[derive(Hash)]
pub struct Likes(pub u32);

#[int(u8, 1..=64)]
pub enum Limit {}

impl Default for Limit {
    fn default() -> Self {
        Self::POS24
    }
}

#[data(copy, display(name))]
#[derive(Default)]
pub enum Sort {
    /// Newest first.
    #[default]
    Recent,
    /// Most liked first, newest first on ties.
    Top,
}

#[data]
pub struct Comment {
    pub id: Id,
    /// `None` for tombstones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<user::Id>,
    /// `None` for tombstones.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Text>,
    pub at: Timestamp,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<Timestamp>,
    pub likes: Likes,
    /// Direct replies count; always 0 for replies themselves.
    pub replies: u32,
    /// Whether the caller liked this comment; `false` when unauthenticated.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub own_liked: bool,
    pub deleted: bool,
}
