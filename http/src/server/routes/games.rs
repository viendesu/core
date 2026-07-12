use super::*;

use crate::requests::{blogs::Get as GetBlog, games::Get};

use viendesu_core::service::{blogs::Blogs, games::Games};
use viendesu_protocol::{
    requests::{
        blogs as blog_reqs,
        games::{create, get, search, update},
    },
    types::{author, game},
};

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/",
            post(async |mut session: SessionOf<T>, ctx: Ctx<create::Args>| {
                session.games().create().call(ctx.request).await
            }),
        )
        .route(
            "/{game_id}",
            patch(
                async |mut session: SessionOf<T>, mut ctx: Ctx<update::Update>| {
                    let id: game::Id = ctx.path().await?;

                    session
                        .games()
                        .update()
                        .call(update::Args {
                            id,
                            update: ctx.request,
                        })
                        .await
                },
            ),
        )
        .route(
            "/{game_id}",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<Get>| {
                let game_id: game::Id = ctx.path().await?;
                let Get {
                    resolve_marks,
                    latest_articles,
                } = ctx.request;

                session
                    .games()
                    .get()
                    .call(get::Args {
                        game: game_id.into(),
                        resolve_marks,
                        latest_articles,
                    })
                    .await
            }),
        )
        .route(
            "/{author}/{slug}",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<Get>| {
                let (author, slug) = ctx.path::<(author::Selector, game::Slug)>().await?;
                let Get {
                    resolve_marks,
                    latest_articles,
                } = ctx.request;

                session
                    .games()
                    .get()
                    .call(get::Args {
                        resolve_marks,
                        latest_articles,
                        game: game::Selector::FullyQualified(game::FullyQualified { author, slug }),
                    })
                    .await
            }),
        )
        .route(
            "/{game_id}/blog",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<GetBlog>| {
                let game_id: game::Id = ctx.path().await?;
                let GetBlog {} = ctx.request;

                session
                    .blogs()
                    .get()
                    .call(blog_reqs::get::Args {
                        blog: game::Selector::from(game_id).into(),
                    })
                    .await
            }),
        )
        .route(
            "/{author}/{slug}/blog",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<GetBlog>| {
                let (author, slug) = ctx.path::<(author::Selector, game::Slug)>().await?;
                let GetBlog {} = ctx.request;

                session
                    .blogs()
                    .get()
                    .call(blog_reqs::get::Args {
                        blog: game::Selector::FullyQualified(game::FullyQualified {
                            author,
                            slug,
                        })
                        .into(),
                    })
                    .await
            }),
        )
        .route(
            "/search",
            post(async |mut session: SessionOf<T>, ctx: Ctx<search::Args>| {
                session.games().search().call(ctx.request).await
            }),
        )
}
