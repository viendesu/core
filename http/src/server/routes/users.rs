use super::*;

use crate::requests::{
    articles::Get as GetArticle,
    blogs::Get as GetBlog,
    users::{ConfirmSignUp, FinishAuth, Get},
};
use viendesu_core::service::{articles::Articles, blogs::Blogs, tabs::Tabs, users::Users};
use viendesu_protocol::{
    requests::{
        articles as article_reqs, blogs as blog_reqs,
        users::{
            begin_auth, check_auth, confirm_sign_up, finish_auth, get, search, sign_in, sign_up,
            update,
        },
    },
    types::article,
};

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/",
            get(async |mut session: SessionOf<T>, ctx: Ctx<search::Args>| {
                session.users().search().call(ctx.request).await
            }),
        )
        .route(
            "/{sel}",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<Get>| {
                let selector: user::Selector = ctx.path().await?;
                session
                    .users()
                    .get()
                    .call(get::Args {
                        user: Some(selector),
                    })
                    .await
            }),
        )
        .route(
            "/{sel}/blog",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<GetBlog>| {
                let selector: user::Selector = ctx.path().await?;
                let GetBlog {} = ctx.request;
                session
                    .blogs()
                    .get()
                    .call(blog_reqs::get::Args {
                        blog: selector.into(),
                    })
                    .await
            }),
        )
        .route(
            "/{sel}/blog/articles/{article_sel}",
            get(
                async |mut session: SessionOf<T>, mut ctx: Ctx<GetArticle>| {
                    let (user, article) =
                        ctx.path::<(user::Selector, article::Selector)>().await?;
                    let GetArticle {} = ctx.request;
                    session
                        .articles()
                        .get()
                        .call(article_reqs::get::Args {
                            article,
                            blog: Some(user.into()),
                        })
                        .await
                },
            ),
        )
        .route(
            "/begin-auth",
            post(
                async |mut session: SessionOf<T>, ctx: Ctx<begin_auth::Args>| {
                    session.users().begin_auth().call(ctx.request).await
                },
            ),
        )
        .route(
            "/finish-auth/{authsessid}",
            post(
                async |mut session: SessionOf<T>, mut ctx: Ctx<FinishAuth>| {
                    let auth_session: user::AuthSessionId = ctx.path().await?;
                    session
                        .users()
                        .finish_auth()
                        .call(finish_auth::Args { auth_session })
                        .await
                },
            ),
        )
        .route(
            "/check-auth",
            get(
                async |mut session: SessionOf<T>, ctx: Ctx<check_auth::Args>| {
                    session.users().check_auth().call(ctx.request).await
                },
            ),
        )
        .route(
            "/me",
            get(async |mut session: SessionOf<T>, _ctx: Ctx<Get>| {
                session.users().get().call(get::Args { user: None }).await
            }),
        )
        .route(
            "/sign_in",
            post(async |mut session: SessionOf<T>, ctx: Ctx<sign_in::Args>| {
                session.users().sign_in().call(ctx.request).await
            }),
        )
        .route(
            "/sign_up",
            post(async |mut session: SessionOf<T>, ctx: Ctx<sign_up::Args>| {
                session.users().sign_up().call(ctx.request).await
            }),
        )
        .route(
            "/{sel}",
            patch(
                async |mut session: SessionOf<T>, mut ctx: Ctx<update::Update>| {
                    let user: user::Selector = ctx.path().await?;
                    session
                        .users()
                        .update()
                        .call(update::Args {
                            user: Some(user),
                            update: ctx.request,
                        })
                        .await
                },
            ),
        )
        .route(
            "/me",
            patch(
                async |mut session: SessionOf<T>, ctx: Ctx<update::Update>| {
                    session
                        .users()
                        .update()
                        .call(update::Args {
                            user: None,
                            update: ctx.request,
                        })
                        .await
                },
            ),
        )
        .route(
            "/{user}/confirm/{token}",
            post(
                async |mut session: SessionOf<T>, mut ctx: Ctx<ConfirmSignUp>| {
                    let (user, token) = ctx.path().await?;
                    session
                        .users()
                        .confirm_sign_up()
                        .call(confirm_sign_up::Args { user, token })
                        .await
                },
            ),
        )
        .nest("/{user}/tabs", |router| {
            use crate::requests::tabs::{Delete, Insert, List, ListItems};
            use viendesu_protocol::requests::tabs::{delete, insert, list, list_items};

            router
                .route(
                    "/",
                    get(async |mut session: SessionOf<T>, mut ctx: Ctx<List>| {
                        let user = ctx.path().await?;
                        let List {} = ctx.request;
                        session.tabs().list().call(list::Args { user }).await
                    }),
                )
                .nest("/{tab}", |router| {
                    router
                        .route(
                            "/{item}",
                            delete(async |mut session: SessionOf<T>, mut ctx: Ctx<Delete>| {
                                let (user, tab, item) = ctx.path().await?;
                                let Delete {} = ctx.request;

                                session
                                    .tabs()
                                    .delete()
                                    .call(delete::Args { user, tab, item })
                                    .await
                            }),
                        )
                        .route(
                            "/",
                            post(async |mut session: SessionOf<T>, mut ctx: Ctx<Insert>| {
                                let (user, tab) = ctx.path().await?;
                                let Insert { item } = ctx.request;

                                session
                                    .tabs()
                                    .insert()
                                    .call(insert::Args { user, tab, item })
                                    .await
                            }),
                        )
                        .route(
                            "/",
                            get(async |mut session: SessionOf<T>, mut ctx: Ctx<ListItems>| {
                                let (user, tab) = ctx.path().await?;
                                let ListItems {
                                    resolve_marks,
                                    start_from,
                                    limit,
                                } = ctx.request;

                                session
                                    .tabs()
                                    .list_items()
                                    .call(list_items::Args {
                                        tab,
                                        user,
                                        start_from,
                                        limit,
                                        resolve_marks,
                                    })
                                    .await
                            }),
                        )
                })
        })
}
