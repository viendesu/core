use super::*;

use crate::requests::{articles::Get as GetArticle, authors::Get, blogs::Get as GetBlog};

use viendesu_core::service::{articles::Articles, authors::Authors, blogs::Blogs};
use viendesu_protocol::{
    requests::{
        articles as article_reqs,
        authors::{create, get, search, update},
        blogs as blog_reqs,
    },
    types::{article, author},
};

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/",
            get(async |mut session: SessionOf<T>, ctx: Ctx<search::Args>| {
                session.authors().search().call(ctx.request).await
            }),
        )
        .route(
            "/",
            post(async |mut session: SessionOf<T>, ctx: Ctx<create::Args>| {
                session.authors().create().call(ctx.request).await
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
            "/{selector}/blog",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<GetBlog>| {
                let author: author::Selector = ctx.path().await?;
                let GetBlog {} = ctx.request;

                session
                    .blogs()
                    .get()
                    .call(blog_reqs::get::Args {
                        blog: author.into(),
                    })
                    .await
            }),
        )
        .route(
            "/{selector}/blog/articles/{article_sel}",
            get(
                async |mut session: SessionOf<T>, mut ctx: Ctx<GetArticle>| {
                    let (author, article) =
                        ctx.path::<(author::Selector, article::Selector)>().await?;
                    let GetArticle {} = ctx.request;

                    session
                        .articles()
                        .get()
                        .call(article_reqs::get::Args {
                            article,
                            blog: Some(author.into()),
                        })
                        .await
                },
            ),
        )
        .route(
            "/{selector}",
            patch(
                async |mut session: SessionOf<T>, mut ctx: Ctx<update::Update>| {
                    let author: author::Selector = ctx.path().await?;

                    session
                        .authors()
                        .update()
                        .call(update::Args {
                            author,
                            update: ctx.request,
                        })
                        .await
                },
            ),
        )
}
