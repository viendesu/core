use super::*;

use crate::requests::{
    articles::{
        Create as CreateArticle, Delete as DeleteArticle, Edit as EditArticle, Get as GetArticle,
        Search as SearchArticles,
    },
    blogs::{Edit as EditBlog, Get as GetBlog},
};
use viendesu_core::service::{articles::Articles, blogs::Blogs};
use viendesu_protocol::{
    requests::{articles as article_reqs, blogs as blog_reqs},
    types::{article, blog},
};

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/{sel}",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<GetBlog>| {
                let blog: blog::Selector = ctx.path().await?;
                let GetBlog {} = ctx.request;
                session
                    .blogs()
                    .get()
                    .call(blog_reqs::get::Args { blog })
                    .await
            }),
        )
        .route(
            "/{sel}",
            patch(async |mut session: SessionOf<T>, mut ctx: Ctx<EditBlog>| {
                let blog: blog::Selector = ctx.path().await?;
                let EditBlog { title, description } = ctx.request;
                session
                    .blogs()
                    .edit()
                    .call(blog_reqs::edit::Args {
                        blog,
                        title,
                        description,
                    })
                    .await
            }),
        )
        .nest("/{sel}/articles", |router| {
            router
                .route(
                    "/",
                    get(
                        async |mut session: SessionOf<T>, mut ctx: Ctx<SearchArticles>| {
                            let blog: blog::Selector = ctx.path().await?;
                            let SearchArticles { limit, after } = ctx.request;
                            session
                                .articles()
                                .search()
                                .call(article_reqs::search::Args { blog, limit, after })
                                .await
                        },
                    ),
                )
                .route(
                    "/",
                    post(
                        async |mut session: SessionOf<T>, mut ctx: Ctx<CreateArticle>| {
                            let blog: blog::Selector = ctx.path().await?;
                            let CreateArticle { title, content } = ctx.request;
                            session
                                .articles()
                                .create()
                                .call(article_reqs::create::Args {
                                    blog,
                                    title,
                                    content,
                                })
                                .await
                        },
                    ),
                )
        })
}

pub fn articles<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/{sel}",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<GetArticle>| {
                let article: article::Selector = ctx.path().await?;
                let GetArticle {} = ctx.request;
                session
                    .articles()
                    .get()
                    .call(article_reqs::get::Args { article })
                    .await
            }),
        )
        .route(
            "/{sel}",
            patch(
                async |mut session: SessionOf<T>, mut ctx: Ctx<EditArticle>| {
                    let article: article::Id = ctx.path().await?;
                    let EditArticle { title, content } = ctx.request;
                    session
                        .articles()
                        .edit()
                        .call(article_reqs::edit::Args {
                            article,
                            title,
                            content,
                        })
                        .await
                },
            ),
        )
        .route(
            "/{sel}",
            delete(
                async |mut session: SessionOf<T>, mut ctx: Ctx<DeleteArticle>| {
                    let article: article::Id = ctx.path().await?;
                    let DeleteArticle {} = ctx.request;
                    session
                        .articles()
                        .delete()
                        .call(article_reqs::delete::Args { article })
                        .await
                },
            ),
        )
}
