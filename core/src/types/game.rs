use std::collections::HashMap;

use eva::{array, data, int, str, str::CompactString, time::Date, url};

use crate::types::{author, entity::define_eid, file, mark, slug, user};

#[data]
#[derive(Default)]
pub struct Screenshots(pub array::ImmutableHeap<file::Id, 8>);

/// Query for searching.
#[str(newtype)]
pub struct SearchQuery(pub CompactString);

/// Fully qualified game path.
#[data(copy, display("{author}/@{slug}"))]
pub struct FullyQualified {
    pub author: author::Selector,
    pub slug: Slug,
}

/// How game can be selected.
#[data(copy)]
pub enum Selector {
    #[display("{_0}")]
    FullyQualified(#[from] FullyQualified),
    #[display("{_0}")]
    Id(#[from] Id),
}

/// Date precision.
#[data(copy, ord, display("name"))]
pub enum DatePrecision {
    /// Precise to day, month and year.
    Day,
    /// Precise to month and year.
    Month,
    /// Precise to year.
    Year,
}

/// Date when game is released.
#[data]
pub struct ReleaseDate {
    /// Date.
    pub date: Date,
    /// Precision of the date.
    pub precision: DatePrecision,
}

/// ID of the vndb entry.
#[data(copy, ord, display("v{_0}"))]
pub struct VndbId(pub u64);

#[data]
pub struct PubModerated {
    pub by: user::Id,
    pub at: Date,
}

#[data]
pub struct PubVerified {
    pub by: user::Id,
    pub at: Date,
}

#[data]
pub enum Publication {
    /// Game was published after a manual moderation.
    Moderated(PubModerated),
    /// Game was published immediately after publication
    /// request, since author is verified.
    Verified(PubVerified),
}

#[data]
#[derive(Default)]
pub struct Marks {
    pub tags: HashMap<mark::Tag, CompactString>,
    pub badges: HashMap<mark::Badge, CompactString>,
}

#[data(copy)]
pub enum Platform {
    Android,
    Ios,
    Pc {
        linux: bool,
        mac: bool,
        windows: bool,
    },
}

#[data]
pub enum DownloadLink {
    External(url::Url),
    Dedicated(file::Id),
}

#[data]
pub struct Download {
    pub platform: Platform,
    pub link: DownloadLink,
    pub label: CompactString,
}

#[data]
pub struct Game {
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<Slug>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<file::Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vndb: Option<VndbId>,
    pub title: Title,
    pub description: Option<Description>,
    pub mean_rating: MeanRating,
    pub author: author::Mini,
    pub release_date: Option<ReleaseDate>,
    pub publication: Option<Publication>,
    pub downloads: Vec<Download>,

    pub screenshots: Screenshots,
    pub tags: mark::Tags,
    pub genres: mark::Genres,
    pub badges: mark::Badges,
}

#[data(copy)]
pub struct MeanRating {
    /// Mean value of all ratings.
    pub mean: RatingValue,
    pub votes: Votes,
}

impl Default for MeanRating {
    fn default() -> Self {
        Self {
            mean: RatingValue::POS0,
            votes: Votes(0),
        }
    }
}

/// Number of votes from users.
#[data(copy, ord, display("{_0}"))]
#[derive(Hash)]
pub struct Votes(pub u32);

define_eid! {
    /// ID of the game.
    pub struct Id(Game);
}

/// Estimated time to read.
#[int(u8, 0..=100)]
pub enum TimeToReadHours {}

/// Numerical rating of the game.
#[int(u8, 0..=100)]
pub enum RatingValue {}

/// Short, url-safe human-readable identifier of the game.
#[str(newtype, copy)]
pub struct Slug(pub slug::Slug<31>);

/// Game title.
#[str(newtype)]
pub struct Title(CompactString);

/// Game description.
#[str(newtype)]
pub struct Description(CompactString);
