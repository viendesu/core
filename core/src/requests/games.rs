use crate::{
    errors,
    types::{Patch, True, author, file, game, mark},
};

use eva::{array, data, int, str, time};

pub mod update {
    use super::*;

    #[data]
    pub struct Args {
        pub id: game::Id,
        pub update: Update,
    }

    #[data]
    #[serde_with::apply(Patch => #[serde(default)])]
    pub struct Update {
        pub title: Patch<game::Title>,
        pub description: Patch<Option<game::Description>>,
        pub slug: Patch<game::Slug>,
        pub thumbnail: Patch<Option<file::Id>>,
        pub genres: Patch<mark::Genres>,
        pub downloads: Patch<Vec<game::Download>>,
        pub badges: Patch<mark::Badges>,
        pub tags: Patch<mark::Tags>,
        pub screenshots: Patch<game::Screenshots>,
        pub published: Patch<bool>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::games::NotFound),
        #[display("{_0}")]
        NoSuchTag(#[from] errors::marks::NoSuchTag),
        #[display("{_0}")]
        NoSuchBadge(#[from] errors::marks::NoSuchBadge),
        #[display("{_0}")]
        NoSuchGenre(#[from] errors::marks::NoSuchGenre),
        #[display("{_0}")]
        UnexpectedFileClass(#[from] errors::files::UnexpectedFileClass),
        #[display("{_0}")]
        InvalidImage(#[from] errors::files::InvalidImage),
    }
}

pub mod search {
    use super::*;

    type Arr<T> = array::ImmutableHeap<T, 16>;

    #[data]
    #[derive(Default)]
    pub struct Marks {
        #[serde(default)]
        pub tags_all: Arr<mark::Tag>,
        #[serde(default)]
        pub tags_any: Arr<mark::Tag>,

        #[serde(default)]
        pub badges_all: Arr<mark::Badge>,
        #[serde(default)]
        pub badges_any: Arr<mark::Badge>,

        #[serde(default)]
        pub genres_all: Arr<mark::Genre>,
        #[serde(default)]
        pub genres_any: Arr<mark::Genre>,
    }

    type SortKey<K> = (K, game::Id);

    #[data(copy, display(name))]
    pub enum SortBy {
        /// By game id. Simplest possible ordering.
        Id { after: Option<game::Id> },
        /// By game release date.
        ReleaseDate { after: Option<SortKey<time::Date>> },
        /// By publish on site date.
        PublishedAt { after: Option<SortKey<time::Date>> },
        /// By game rating.
        Rating {
            after: Option<SortKey<game::RatingValue>>,
        },
    }

    impl Default for SortBy {
        fn default() -> Self {
            Self::Id { after: None }
        }
    }

    #[data(copy, display(name))]
    #[derive(Default)]
    pub enum Order {
        /// From lowest to highest.
        Asc,
        /// From highest to lowest.
        #[default]
        Desc,
    }

    #[int(u8, 1..=32)]
    pub enum Limit {}

    impl Default for Limit {
        fn default() -> Self {
            Self::POS16
        }
    }

    #[data]
    pub struct Args {
        pub query: Option<game::SearchQuery>,
        pub author: Option<author::Selector>,
        #[serde(default)]
        pub include: Marks,
        #[serde(default)]
        pub exclude: Marks,
        #[serde(default)]
        pub order: Order,
        #[serde(default)]
        pub sort_by: SortBy,
        pub limit: Option<Limit>,
    }

    #[data]
    pub struct Ok {
        pub found: Vec<game::Game>,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchAuthor(#[from] errors::authors::NotFound),
    }
}

pub mod create {
    use super::*;

    #[data]
    pub struct Args {
        pub title: game::Title,
        pub description: Option<game::Description>,
        pub thumbnail: Option<file::Id>,
        pub author: author::Id,
        #[serde(default)]
        pub tags: mark::Tags,
        #[serde(default)]
        pub screenshots: game::Screenshots,
        #[serde(default)]
        pub genres: mark::Genres,
        #[serde(default)]
        pub downloads: Vec<game::Download>,
        pub slug: Option<game::Slug>,
        pub vndb: Option<game::VndbId>,
        pub release_date: Option<game::ReleaseDate>,
    }

    #[data]
    pub struct Ok {
        pub id: game::Id,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NoSuchAuthor(#[from] errors::authors::NotFound),
        #[display("{_0}")]
        AlreadyTaken(#[from] errors::games::AlreadyTaken),
        #[display("{_0}")]
        NoSuchTag(#[from] errors::marks::NoSuchTag),
        #[display("{_0}")]
        NoSuchGenre(#[from] errors::marks::NoSuchGenre),
        #[display("{_0}")]
        UnexpectedFileClass(#[from] errors::files::UnexpectedFileClass),
        #[display("{_0}")]
        InvalidImage(#[from] errors::files::InvalidImage),
    }
}

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub game: game::Selector,
        pub resolve_marks: bool,
    }

    #[data]
    pub struct Ok {
        // TODO: include if requested
        // - translation maps.
        // - comments
        // - downloads
        pub game: game::Game,
        pub marks: game::Marks,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::games::NotFound),
        #[display("{_0}")]
        NoSuchAuthor(#[from] errors::authors::NotFound),
    }
}
