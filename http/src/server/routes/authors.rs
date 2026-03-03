use super::*;

use crate::requests::authors::{Create, Get, Search, Update};

use viendesu_core::{
    requests::authors::{create, get, search, update},
    service::authors::Authors,
    types::author,
};

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/",
            get(async |mut session: SessionOf<T>, ctx: Ctx<Search>| {
                let Search {
                    query,
                    owned_by,
                    start_from,
                    limit,
                } = ctx.request;
                session
                    .authors()
                    .search()
                    .call(search::Args {
                        query,
                        start_from,
                        owned_by,
                        limit,
                    })
                    .await
            }),
        )
        .route(
            "/",
            post(async |mut session: SessionOf<T>, ctx: Ctx<Create>| {
                let Create {
                    title,
                    slug,
                    description,
                    owner,
                    pfp,
                } = ctx.request;
                session
                    .authors()
                    .create()
                    .call(create::Args {
                        title,
                        slug,
                        description,
                        pfp,
                        owner,
                    })
                    .await
            }),
        )
        .route(
            "/{selector}",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<Get>| {
                let author: author::Selector = ctx.path().await?;
                let Get {} = ctx.request;

                session.authors().get().call(get::Args { author }).await
            }),
        )
        .route(
            "/{selector}",
            patch(async |mut session: SessionOf<T>, mut ctx: Ctx<Update>| {
                let author: author::Selector = ctx.path().await?;
                let Update {
                    title,
                    description,
                    pfp,
                    slug,
                    verified,
                } = ctx.request;

                session
                    .authors()
                    .update()
                    .call(update::Args {
                        author,
                        update: update::Update {
                            title,
                            description,
                            pfp,
                            slug,
                            verified,
                        },
                    })
                    .await
            }),
        )
}
