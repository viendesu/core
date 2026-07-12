use eva::{data, str};

use crate::types::{author, entity, game, user};

entity::define_eid! {
    /// Identifier of a blog: the id of the entity that authors it. A blog is
    /// not a separate entity, it's a facet of its owner, so the owner kind is
    /// readable straight from the id.
    pub struct Id(User | Author | Game);
}

impl Id {
    pub const fn user(user: user::Id) -> Self {
        Self(user.raw_id())
    }

    pub const fn author(author: author::Id) -> Self {
        Self(author.raw_id())
    }

    pub const fn game(game: game::Id) -> Self {
        Self(game.raw_id())
    }
}

/// The game whose blog it is, along with the author that owns the game:
/// the author's `owner` is the user allowed to edit the blog.
#[data]
pub struct GameOwner {
    pub game: game::Mini,
    pub author: author::Mini,
}

/// Resolved owner of a blog. The kind matches the kind of the blog id.
#[data]
pub enum Owner {
    User(user::Mini),
    Author(author::Mini),
    Game(GameOwner),
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
