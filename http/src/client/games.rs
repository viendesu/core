use super::*;

use viendesu_core::{
    requests::games::{create, get, search, update},
    types::game::Selector,
};

use crate::requests::games as requests;

impl Games for HttpClient {
    fn get(&mut self) -> impl CallStep<get::Args, Ok = get::Ok, Err = get::Err> {
        self.do_call(
            Method::GET,
            |get::Args {
                 game,
                 resolve_marks,
             }| match game {
                Selector::Id(id) => (
                    c!("/games/{}", id.to_str()),
                    requests::Get { resolve_marks },
                ),
                Selector::FullyQualified(fq) => (
                    c!("/games/{}/{}", fq.author, fq.slug),
                    requests::Get { resolve_marks },
                ),
            },
        )
    }

    fn search(&mut self) -> impl CallStep<search::Args, Ok = search::Ok, Err = search::Err> {
        self.do_call(
            Method::POST,
            |search::Args {
                 query,
                 author,
                 include,
                 exclude,
                 order,
                 sort_by,
                 limit,
             }| {
                (
                    "/games/search".into(),
                    requests::Search {
                        query,
                        author,
                        include,
                        exclude,
                        order,
                        sort_by,
                        limit,
                    },
                )
            },
        )
    }

    fn create(&mut self) -> impl CallStep<create::Args, Ok = create::Ok, Err = create::Err> {
        self.do_call(
            Method::POST,
            |create::Args {
                 title,
                 description,
                 thumbnail,
                 downloads,
                 author,
                 slug,
                 vndb,
                 release_date,
                 tags,
                 genres,
                 screenshots,
             }| {
                (
                    "/games".into(),
                    requests::Create {
                        title,
                        description,
                        thumbnail,
                        screenshots,
                        author,
                        tags,
                        downloads,
                        genres,
                        slug,
                        vndb,
                        release_date,
                    },
                )
            },
        )
    }

    fn update(&mut self) -> impl CallStep<update::Args, Ok = update::Ok, Err = update::Err> {
        self.do_call(Method::PATCH, |update::Args { id, update }| {
            (
                c!("/games/{id}"),
                requests::Update {
                    downloads: update.downloads,
                    title: update.title,
                    description: update.description,
                    slug: update.slug,
                    thumbnail: update.thumbnail,
                    genres: update.genres,
                    badges: update.badges,
                    tags: update.tags,
                    screenshots: update.screenshots,
                    published: update.published,
                },
            )
        })
    }
}
