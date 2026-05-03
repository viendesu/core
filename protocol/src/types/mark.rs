use eva::{array, data, str};

use crate::types::{entity, slug};

#[data]
#[derive(Default)]
pub struct Tags(pub array::ImmutableHeap<Tag, 64>);

#[data]
#[derive(Default)]
pub struct Genres(pub array::ImmutableHeap<Genre, 64>);

#[data]
#[derive(Default)]
pub struct Badges(pub array::ImmutableHeap<Badge, 64>);

#[str(newtype, copy)]
pub struct Genre(pub slug::LowerSlug<15>);

entity::define_eid! {
    /// ID of the tag.
    pub struct Tag(Tag);

    /// Game badge.
    pub struct Badge(Badge);
}
