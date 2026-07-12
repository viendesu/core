use eva::{data, str};

use crate::types::{author, entity, game, user};

entity::define_eid! {
    pub struct Id(Blog);
}

entity::define_eid! {
    pub struct OwnerId(User | Author | Game);
}

impl OwnerId {
    pub const fn user(user: user::Id) -> Self {
        Self(user.raw_id())
    }

    pub const fn author(author: author::Id) -> Self {
        Self(author.raw_id())
    }

    pub const fn game(game: game::Id) -> Self {
        Self(game.raw_id())
    }

    /// Blog of this entity: the same identifier with the kind swapped to
    /// [`entity::Kind::Blog`], everything else (data bits included) preserved.
    pub const fn blog(self) -> Id {
        let raw = self.raw_id();
        let meta = entity::Metadata::new(entity::Kind::Blog, raw.metadata().data());

        match Id::from_generic(raw.with_metadata(meta)) {
            Some(id) => id,
            None => unreachable!(),
        }
    }
}

#[data(copy)]
#[serde(untagged)]
pub enum Selector {
    /// The blog id itself.
    #[display("{_0}")]
    Id(#[from] Id),
    /// Id of the owning entity, the blog id is derived from it.
    #[display("{_0}")]
    Owner(#[from] OwnerId),
}

impl Selector {
    /// Normalize to the blog id.
    pub const fn id(self) -> Id {
        match self {
            Self::Id(id) => id,
            Self::Owner(owner) => owner.blog(),
        }
    }
}

#[str(newtype)]
pub struct Title(str::CompactString);

/// Blog's description.
#[str(newtype)]
pub struct Description(pub str::CompactString);

#[data]
pub struct Blog {
    pub id: Id,
    pub title: Option<Title>,
    pub description: Option<Description>,
}
