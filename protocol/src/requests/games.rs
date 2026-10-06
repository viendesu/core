use std::collections::HashMap;

use crate::{
    errors,
    types::{Patch, True, article, author, comment, file, game, mark, user},
};

use eva::{array, data, int, str, time};

pub mod rate {
    use super::*;

    #[data]
    pub struct Args {
        pub id: game::Id,
        /// `None` retracts the caller's vote.
        pub rating: Option<game::RatingValue>,
    }

    #[data]
    pub struct Ok {
        /// Rating of the game after the vote.
        pub rating: game::MeanRating,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::games::NotFound),
    }
}

pub mod update {
    use super::*;

    #[data]
    pub struct Args {
        pub id: game::Id,
        pub update: Update,
    }

    #[serde_with::apply(Patch => #[serde(default)])]
    #[data]
    pub struct Update {
        pub title: Patch<game::Title>,
        pub alt_titles: Patch<game::AltTitles>,
        pub description: Patch<Option<game::Description>>,
        pub slug: Patch<game::Slug>,
        pub thumbnail: Patch<Option<file::Id>>,
        pub vndb: Patch<Option<game::VndbId>>,
        pub genres: Patch<mark::Genres>,
        pub releases: Patch<game::Releases>,
        pub badges: Patch<mark::Badges>,
        pub tags: Patch<mark::Tags>,
        pub screenshots: Patch<game::Screenshots>,
        pub published: Patch<bool>,
        /// Replaces the relations the caller can see; ones to games hidden
        /// from the caller are kept.
        pub related: Patch<game::Related>,
    }

    #[data]
    pub struct Ok(pub True);

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::games::NotFound),
        #[display("{_0}")]
        AlreadyTaken(#[from] errors::games::AlreadyTaken),
        #[display("{_0}")]
        NoSuchTag(#[from] errors::tags::NoSuchTag),
        #[display("{_0}")]
        NoSuchBadge(#[from] errors::badges::NoSuchBadge),
        #[display("{_0}")]
        NoSuchGenre(#[from] errors::genres::NoSuchGenre),
        #[display("{_0}")]
        UnexpectedFileClass(#[from] errors::files::UnexpectedFileClass),
        #[display("{_0}")]
        InvalidImage(#[from] errors::files::InvalidImage),
        #[display("{_0}")]
        FileNotFound(#[from] errors::files::NotFound),
        #[display("{_0}")]
        BadRelation(#[from] errors::games::BadRelation),
        #[display("{_0}")]
        TooManyRelations(#[from] errors::games::TooManyRelations),
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

        /// Matched against the platforms of the game's 4 newest releases.
        #[serde(default)]
        pub platforms_all: game::Platforms,
        /// Matched against the platforms of the game's 4 newest releases.
        #[serde(default)]
        pub platforms_any: game::Platforms,
    }

    type SortKey<K> = (K, game::Id);

    #[data(copy, display(name))]
    pub enum SortBy {
        /// By game id. Simplest possible ordering.
        Id { after: Option<game::Id> },
        /// By game release date.
        ReleaseDate { after: Option<SortKey<time::Date>> },
        /// By publish on site date.
        PublishedAt {
            after: Option<SortKey<time::Timestamp>>,
        },
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
        #[serde(default)]
        pub resolve_marks: bool,
    }

    #[data]
    pub struct Ok {
        pub found: Vec<game::Game>,
        /// Tag and badge names of the found games; empty unless requested
        /// via `resolve_marks`.
        #[serde(default)]
        pub marks: game::Marks,
        pub authors: HashMap<author::Id, author::Mini>,
        pub users: HashMap<user::Id, user::Mini>,
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
        #[serde(default)]
        pub alt_titles: game::AltTitles,
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
        pub releases: game::Releases,
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
        NoSuchTag(#[from] errors::tags::NoSuchTag),
        #[display("{_0}")]
        NoSuchGenre(#[from] errors::genres::NoSuchGenre),
        #[display("{_0}")]
        UnexpectedFileClass(#[from] errors::files::UnexpectedFileClass),
        #[display("{_0}")]
        InvalidImage(#[from] errors::files::InvalidImage),
        #[display("{_0}")]
        FileNotFound(#[from] errors::files::NotFound),
    }
}

pub mod get {
    use super::*;

    #[data]
    pub struct Args {
        pub game: game::Selector,
        pub resolve_marks: bool,
        /// Also fetch the few latest articles of the game's blog.
        #[serde(default)]
        pub latest_articles: bool,
        /// Also fetch the first page of the game's top-level comments.
        #[serde(default)]
        pub comments: bool,
        /// Also fetch the related games.
        #[serde(default)]
        pub related: bool,
    }

    #[data]
    pub struct Ok {
        // TODO: include if requested
        // - translation maps.
        // - releases
        pub game: game::Game,
        /// The caller's own rating of the game, if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub own_rating: Option<game::RatingValue>,
        pub marks: game::Marks,
        pub authors: HashMap<author::Id, author::Mini>,
        pub users: HashMap<user::Id, user::Mini>,
        /// Latest articles of the game's blog, newest first.
        /// Empty unless requested via `latest_articles`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub latest_articles: Vec<article::Mini>,
        /// First page of the game's top-level comments, most liked first;
        /// continue via `comments::list` with `before` + `before_likes`.
        /// Their authors are merged into `users`. Empty unless requested
        /// via `comments`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub comments: Vec<comment::Comment>,
        /// Relations to the games visible to the caller; their authors are
        /// merged into `authors`. Empty unless requested via `related`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub related: Vec<game::Relation>,
        /// Miniatures of the games in `related`.
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        pub games: HashMap<game::Id, game::Mini>,
    }

    #[data(error)]
    pub enum Err {
        #[display("{_0}")]
        NotFound(#[from] errors::games::NotFound),
        #[display("{_0}")]
        NoSuchAuthor(#[from] errors::authors::NotFound),
    }
}
