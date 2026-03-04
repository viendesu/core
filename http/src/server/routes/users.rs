use super::*;

use crate::requests::users::{
    BeginAuth, CheckAuth, ConfirmSignUp, FinishAuth, Get, Search, SignIn, SignUp, Update,
};
use viendesu_core::{
    requests::users::{
        begin_auth, check_auth, confirm_sign_up, finish_auth, get, search, sign_in, sign_up, update,
    },
    service::{tabs::Tabs, users::Users},
};

fn convert_update(u: Update) -> update::Update {
    update::Update {
        nickname: u.nickname,
        display_name: u.display_name,
        bio: u.bio,
        password: u.password,
        role: u.role,
        pfp: u.pfp,
        email: u.email,
    }
}

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/",
            get(async |mut session: SessionOf<T>, ctx: Ctx<Search>| {
                let Search {
                    query,
                    limit,
                    start_from,
                } = ctx.request;
                session
                    .users()
                    .search()
                    .call(search::Args {
                        query,
                        limit,
                        start_from,
                    })
                    .await
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
            "/begin-auth",
            post(async |mut session: SessionOf<T>, ctx: Ctx<BeginAuth>| {
                session
                    .users()
                    .begin_auth()
                    .call(begin_auth::Args {
                        method: ctx.request.method,
                    })
                    .await
            }),
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
            get(async |mut session: SessionOf<T>, _ctx: Ctx<CheckAuth>| {
                session.users().check_auth().call(check_auth::Args {}).await
            }),
        )
        .route(
            "/me",
            get(async |mut session: SessionOf<T>, _ctx: Ctx<Get>| {
                session.users().get().call(get::Args { user: None }).await
            }),
        )
        .route(
            "/sign_in",
            post(async |mut session: SessionOf<T>, ctx: Ctx<SignIn>| {
                session
                    .users()
                    .sign_in()
                    .call(sign_in::Args {
                        nickname: ctx.request.nickname,
                        password: ctx.request.password,
                    })
                    .await
            }),
        )
        .route(
            "/sign_up",
            post(async |mut session: SessionOf<T>, ctx: Ctx<SignUp>| {
                let SignUp {
                    nickname,
                    email,
                    password,
                    display_name,
                } = ctx.request;
                session
                    .users()
                    .sign_up()
                    .call(sign_up::Args {
                        nickname,
                        email,
                        display_name,
                        password,
                    })
                    .await
            }),
        )
        .route(
            "/{sel}",
            patch(async |mut session: SessionOf<T>, mut ctx: Ctx<Update>| {
                let user: user::Selector = ctx.path().await?;
                session
                    .users()
                    .update()
                    .call(update::Args {
                        user: Some(user),
                        update: convert_update(ctx.request),
                    })
                    .await
            }),
        )
        .route(
            "/me",
            patch(async |mut session: SessionOf<T>, ctx: Ctx<Update>| {
                session
                    .users()
                    .update()
                    .call(update::Args {
                        user: None,
                        update: convert_update(ctx.request),
                    })
                    .await
            }),
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
            use viendesu_core::requests::tabs::{delete, insert, list, list_items};

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
