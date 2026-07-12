use eva::{data, str, time::Timestamp};

use crate::types::{blog, entity};

entity::define_eid! {
    pub struct Id(Article);
}

#[data(copy)]
#[serde(untagged)]
pub enum Selector {
    #[display("{_0}")]
    Id(#[from] Id),
}

#[str(newtype)]
pub struct Title(str::CompactString);

/// CommonMark source of the article body. Stored and served as is,
/// rendering is up to the client.
#[str(newtype)]
pub struct Content(str::CompactString);

#[data]
pub struct Brief {
    pub id: Id,
    pub blog: blog::Id,
    pub title: Title,
    pub created_at: Timestamp,
}

#[data]
pub struct Article {
    pub id: Id,
    pub blog: blog::Id,
    pub title: Title,
    pub content: Content,
    pub created_at: Timestamp,
    pub edited_at: Option<Timestamp>,
}
