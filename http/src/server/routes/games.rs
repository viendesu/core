use super::*;

use crate::requests::games::{Create, Get, Search, Update};

use viendesu_core::{
    requests::games::{create, get, search, update},
    service::games::Games,
    types::{author, game},
};

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/",
            post(async |mut session: SessionOf<T>, ctx: Ctx<Create>| {
                let Create {
                    title,
                    description,
                    thumbnail,
                    tags,
                    genres,
                    author,
                    slug,
                    downloads,
                    vndb,
                    release_date,
                    screenshots,
                } = ctx.request;
                session
                    .games()
                    .create()
                    .call(create::Args {
                        title,
                        description,
                        thumbnail,
                        downloads,
                        screenshots,
                        author,
                        slug,
                        tags,
                        genres,
                        vndb,
                        release_date,
                    })
                    .await
            }),
        )
        .route(
            "/{game_id}",
            patch(async |mut session: SessionOf<T>, mut ctx: Ctx<Update>| {
                let game_id: game::Id = ctx.path().await?;
                let Update {
                    title,
                    description,
                    slug,
                    thumbnail,
                    downloads,
                    genres,
                    badges,
                    tags,
                    screenshots,
                    published,
                } = ctx.request;

                session
                    .games()
                    .update()
                    .call(update::Args {
                        id: game_id,
                        update: update::Update {
                            title,
                            description,
                            slug,
                            thumbnail,
                            genres,
                            downloads,
                            badges,
                            tags,
                            screenshots,
                            published,
                        },
                    })
                    .await
            }),
        )
        .route(
            "/{game_id}",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<Get>| {
                let game_id: game::Id = ctx.path().await?;
                let Get { resolve_marks } = ctx.request;

                session
                    .games()
                    .get()
                    .call(get::Args {
                        game: game_id.into(),
                        resolve_marks,
                    })
                    .await
            }),
        )
        .route(
            "/{author}/{slug}",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<Get>| {
                let (author, slug) = ctx.path::<(author::Selector, game::Slug)>().await?;
                let Get { resolve_marks } = ctx.request;

                session
                    .games()
                    .get()
                    .call(get::Args {
                        resolve_marks,
                        game: game::Selector::FullyQualified(game::FullyQualified { author, slug }),
                    })
                    .await
            }),
        )
        .route(
            "/search",
            post(async |mut session: SessionOf<T>, ctx: Ctx<Search>| {
                let Search {
                    query,
                    author,
                    include,
                    exclude,
                    order,
                    sort_by,
                    limit,
                } = ctx.request;

                session
                    .games()
                    .search()
                    .call(search::Args {
                        query,
                        author,
                        include,
                        exclude,
                        order,
                        sort_by,
                        limit,
                    })
                    .await
            }),
        )
}
