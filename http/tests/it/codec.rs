//! MessagePack carries exactly the JSON shapes (see `Format`).

use std::{collections::HashMap, fmt::Debug};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use eva::{
    array::ImmutableHeap,
    str::CompactString,
    time::{Date, Timestamp},
};

use viendesu_http::format::{DumpParams, Format};
use viendesu_protocol::{
    errors::{self, Aux, Generic},
    requests::{comments, games, users},
    types::{Patch, author, comment, entity, file, game, mark, user},
};

fn raw(kind: entity::Kind, n: u128) -> entity::Id {
    entity::Id::from_parts(1_700_000, n, entity::Metadata::new(kind, 0))
}

fn game_id() -> game::Id {
    game::Id::from_generic(raw(entity::Kind::Game, 7)).unwrap()
}

fn user_id(n: u128) -> user::Id {
    user::Id::from_generic(raw(entity::Kind::User, n)).unwrap()
}

fn author_id() -> author::Id {
    author::Id::from_generic(raw(entity::Kind::Author, 11)).unwrap()
}

fn file_id() -> file::Id {
    file::Id::from_generic(raw(entity::Kind::File, 13)).unwrap()
}

fn ts() -> Timestamp {
    Timestamp::from_millis(1_767_225_600_000)
}

fn release_date() -> game::ReleaseDate {
    game::ReleaseDate {
        date: Date::from_days(20_000),
        precision: game::DatePrecision::Month,
    }
}

fn assert_same_shape<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let mut msgpack = Vec::new();
    Format::Msgpack.dump(DumpParams::default(), value, &mut msgpack);

    let tree: Value = rmp_serde::from_slice(&msgpack).unwrap();
    assert_eq!(tree, serde_json::to_value(value).unwrap());

    let decoded: T = Format::Msgpack.load(&msgpack).unwrap();
    assert_eq!(&decoded, value);
}

fn game() -> game::Game {
    game::Game {
        id: game_id(),
        slug: Some("my-game".parse().unwrap()),
        thumbnail: Some(file_id()),
        vndb: Some(game::VndbId(17)),
        title: "My Game".parse().unwrap(),
        description: None,
        mean_rating: game::MeanRating {
            mean: game::RatingValue::POS75,
            votes: game::Votes::new(12),
        },
        author: author_id(),
        release_date: Some(release_date()),
        publication: None,
        downloads: vec![game::Download {
            platform: game::Platform::Pc {
                linux: true,
                mac: false,
                windows: true,
            },
            link: game::DownloadLink::External("https://example.com/a".parse().unwrap()),
            label: CompactString::from("pc"),
        }],
        screenshots: game::Screenshots(ImmutableHeap::try_from(vec![file_id()]).unwrap()),
        tags: mark::Tags::default(),
        genres: mark::Genres(
            ImmutableHeap::try_from(vec!["rpg".parse::<mark::Genre>().unwrap()]).unwrap(),
        ),
        badges: mark::Badges::default(),
    }
}

#[test]
fn protocol_values() {
    assert_same_shape(&game());

    assert_same_shape(&games::get::Args {
        game: game::Selector::FullyQualified(game::FullyQualified {
            author: author::Selector::Slug("acme".parse().unwrap()),
            slug: "my-game".parse().unwrap(),
        }),
        resolve_marks: true,
        latest_articles: false,
        comments: true,
        related: false,
    });
    assert_same_shape(&games::get::Args {
        game: game_id().into(),
        resolve_marks: false,
        latest_articles: true,
        comments: false,
        related: true,
    });

    assert_same_shape(&games::search::Args {
        query: Some("visual novel".parse().unwrap()),
        author: Some(author::Selector::Id(author_id())),
        include: Default::default(),
        exclude: Default::default(),
        order: games::search::Order::Asc,
        sort_by: games::search::SortBy::PublishedAt {
            after: Some((ts(), game_id())),
        },
        limit: None,
        resolve_marks: true,
    });

    assert_same_shape(&games::update::Update {
        title: Patch::Change("New".parse().unwrap()),
        description: Patch::Change(None),
        slug: Patch::Keep,
        thumbnail: Patch::Keep,
        vndb: Patch::Change(Some(game::VndbId(97))),
        genres: Patch::Keep,
        downloads: Patch::Keep,
        badges: Patch::Keep,
        tags: Patch::Keep,
        screenshots: Patch::Keep,
        published: Patch::Change(true),
        related: Patch::Change(game::Related(
            [game::Relation {
                game: game_id(),
                kind: game::RelationKind::SideStory,
            }]
            .try_into()
            .unwrap(),
        )),
    });

    assert_same_shape(&comments::list::Ok {
        comments: vec![comment::Comment {
            id: comment::Id::from_generic(raw(entity::Kind::Comment, 31)).unwrap(),
            author: Some(user_id(3)),
            text: Some("hello".parse().unwrap()),
            at: ts(),
            edited_at: None,
            likes: comment::Likes::new(5),
            replies: 2,
            own_liked: true,
            deleted: false,
        }],
        users: HashMap::from([(
            user_id(3),
            user::Mini {
                id: user_id(3),
                nickname: "nero".parse().unwrap(),
                display_name: None,
                role: user::Role::Admin,
                pfp: Some(file_id()),
            },
        )]),
    });

    let spec: Generic<games::get::Err> = Generic::Spec(
        errors::games::NotFound {
            game: game_id().into(),
        }
        .into(),
    );
    assert_same_shape(&spec);
    assert_same_shape(&Generic::<games::get::Err>::Aux(Aux::Unauthenticated));
    assert_same_shape(&Generic::<users::sign_in::Err>::Spec(
        errors::users::MustCompleteSignUp.into(),
    ));
    assert_same_shape(&Generic::<games::get::Err>::Aux(Aux::Db("boom".into())));
}

#[test]
fn dates_decode_from_both_codecs() {
    let args = games::create::Args {
        title: "T".parse().unwrap(),
        description: None,
        thumbnail: None,
        author: author_id(),
        tags: Default::default(),
        screenshots: Default::default(),
        genres: Default::default(),
        downloads: Vec::new(),
        slug: None,
        vndb: None,
        release_date: Some(release_date()),
    };

    assert_same_shape(&args);

    let mut json = Vec::new();
    Format::Json.dump(DumpParams::default(), &args, &mut json);
    assert_eq!(
        Format::Json.load::<games::create::Args>(&json).unwrap(),
        args
    );
}

#[test]
fn omitted_patch_fields_keep() {
    let empty = rmp_serde::to_vec_named(&serde_json::json!({})).unwrap();
    let update: games::update::Update = Format::Msgpack.load(&empty).unwrap();
    assert_eq!(update.title, Patch::Keep);
    assert_eq!(update.published, Patch::Keep);
}
