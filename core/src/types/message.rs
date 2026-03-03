use eva::{data, int, str, time::Timestamp};

use crate::types::{author, entity, file, thread, user};

#[int(u8, 1..=128)]
pub enum PaginationLimit {}

#[data(copy)]
#[serde(untagged)]
pub enum Selector {
    #[display("{_0}")]
    Id(#[from] Id),
}

entity::define_eid! {
    pub struct Id(Message);
}

entity::define_eid! {
    pub struct ById(User | Author);
}

impl ById {
    pub const fn user(user: user::Id) -> Self {
        Self(user.raw_id())
    }

    pub const fn author(author: author::Id) -> Self {
        Self(author.raw_id())
    }
}

/// Text of the message.
#[str(newtype)]
pub struct Text(str::CompactString);

#[data]
pub struct By {
    pub id: ById,
    pub display_name: str::CompactString,
    pub pfp: file::Id,
}

#[data]
pub struct Message {
    pub id: Id,
    pub by: By,
    pub thread: thread::Id,
    pub text: Text,
    pub created_at: Timestamp,
}
