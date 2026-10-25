use std::collections::HashMap;

use crate::requests::Response;
use eva::{array, data, int, service, str, time};

use crate::{
    errors,
    types::{Patch, True, article, author, comment, file, game, mark, user},
};

/// The game catalog.
#[service(export, result = Response)]
pub trait Games {
    /// Sets the caller's rating, replacing the previous one.
    #[flags(write_op)]
    #[errors(errors::games::NotFound)]
    fn rate(
        id: game::Id,
        /// `None` retracts the caller's vote.
        rating: Option<game::RatingValue>,
    ) -> ok! {
        /// Rating of the game after the vote.
        rating: game::MeanRating,
    };

    /// Patches a game.
    #[flags(write_op)]
    #[errors(
        errors::games::NotFound,
        errors::games::AlreadyTaken,
        errors::tags::NoSuchTag,
        errors::badges::NoSuchBadge,
        errors::genres::NoSuchGenre,
        errors::files::UnexpectedFileClass,
        errors::files::InvalidImage,
        errors::files::NotFound as FileNotFound,
        errors::games::BadRelation,
        errors::games::TooManyRelations,
    )]
    fn update(id: game::Id, update: this![Update]) -> ok!(True);
    embed! {
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
    }

    /// Searches the catalog; the text query matches titles and alternative titles only.
    ///
    /// Paginated via the `after` key of `sort_by`.
    #[errors(errors::authors::NotFound as NoSuchAuthor)]
    fn search(
        query: Option<game::SearchQuery>,
        author: Option<author::Selector>,
        #[serde(default)] include: this![Marks],
        #[serde(default)] exclude: this![Marks],
        #[serde(default)] order: this![Order],
        #[serde(default)] sort_by: this![SortBy],
        limit: Option<this![Limit]>,
        #[serde(default)] resolve_marks: bool,
    ) -> ok! {
        found: Vec<game::Game>,
        /// Tag and badge names of the found games; empty unless requested
        /// via `resolve_marks`.
        #[serde(default)]
        marks: game::Marks,
        authors: HashMap<author::Id, author::Mini>,
        users: HashMap<user::Id, user::Mini>,
    };
    embed! {
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
    }

    /// Adds a game.
    ///
    /// `author` must be owned by the caller unless it has a moderation role.
    #[flags(write_op)]
    #[errors(
        errors::authors::NotFound as NoSuchAuthor,
        errors::games::AlreadyTaken,
        errors::tags::NoSuchTag,
        errors::genres::NoSuchGenre,
        errors::files::UnexpectedFileClass,
        errors::files::InvalidImage,
        errors::files::NotFound as FileNotFound,
    )]
    fn create(
        title: game::Title,
        #[serde(default)] alt_titles: game::AltTitles,
        description: Option<game::Description>,
        thumbnail: Option<file::Id>,
        author: author::Id,
        #[serde(default)] tags: mark::Tags,
        #[serde(default)] screenshots: game::Screenshots,
        #[serde(default)] genres: mark::Genres,
        #[serde(default)] releases: game::Releases,
        slug: Option<game::Slug>,
        vndb: Option<game::VndbId>,
        release_date: Option<game::ReleaseDate>,
    ) -> ok! {
        id: game::Id,
    };

    /// Gets a game.
    #[errors(errors::games::NotFound, errors::authors::NotFound as NoSuchAuthor)]
    fn get(
        game: game::Selector,
        resolve_marks: bool,
        /// Also fetch the few latest articles of the game's blog.
        #[serde(default)]
        latest_articles: bool,
        /// Also fetch the first page of the game's top-level comments.
        #[serde(default)]
        comments: bool,
        /// Also fetch the related games.
        #[serde(default)]
        related: bool,
    ) -> ok! {
        game: game::Game,
        /// The caller's own rating of the game, if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        own_rating: Option<game::RatingValue>,
        marks: game::Marks,
        authors: HashMap<author::Id, author::Mini>,
        users: HashMap<user::Id, user::Mini>,
        /// Latest articles of the game's blog, newest first.
        /// Empty unless requested via `latest_articles`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        latest_articles: Vec<article::Mini>,
        /// First page of the game's top-level comments, most liked first;
        /// continue via `comments::list` with `before` + `before_likes`.
        /// Their authors are merged into `users`. Empty unless requested
        /// via `comments`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        comments: Vec<comment::Comment>,
        /// Relations to the games visible to the caller; their authors are
        /// merged into `authors`. Empty unless requested via `related`.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        related: Vec<game::Relation>,
        /// Miniatures of the games in `related`.
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        games: HashMap<game::Id, game::Mini>,
    };
}
