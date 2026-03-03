use super::*;

pub fn genres<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    use crate::requests::marks::ListGenres;

    use viendesu_core::{requests::marks::list_genres, service::marks::Genres as _};

    router.route(
        "/",
        get(
            async move |mut session: SessionOf<T>, ctx: Ctx<ListGenres>| {
                let ListGenres {} = ctx.request;

                session.genres().list().call(list_genres::Args {}).await
            },
        ),
    )
}

pub fn badges<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    use crate::requests::marks::{AddBadge, ListBadges};

    use viendesu_core::{
        requests::marks::{add_badge, list_badges},
        service::marks::Badges as _,
    };

    router
        .route(
            "/",
            get(
                async move |mut session: SessionOf<T>, ctx: Ctx<ListBadges>| {
                    let ListBadges { query } = ctx.request;
                    session
                        .badges()
                        .list()
                        .call(list_badges::Args { query })
                        .await
                },
            ),
        )
        .route(
            "/",
            post(async move |mut session: SessionOf<T>, ctx: Ctx<AddBadge>| {
                let AddBadge { text } = ctx.request;
                session
                    .badges()
                    .add()
                    .call(add_badge::Args { badge: text })
                    .await
            }),
        )
}

pub fn tags<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    use crate::requests::marks::{AddTag, ListTags};

    use viendesu_core::{
        requests::marks::{add_tag, list_tags},
        service::marks::Tags as _,
    };

    router
        .route(
            "/",
            get(async move |mut session: SessionOf<T>, ctx: Ctx<ListTags>| {
                let ListTags { query } = ctx.request;
                session.tags().list().call(list_tags::Args { query }).await
            }),
        )
        .route(
            "/",
            post(async move |mut session: SessionOf<T>, ctx: Ctx<AddTag>| {
                let AddTag { text } = ctx.request;
                session.tags().add().call(add_tag::Args { tag: text }).await
            }),
        )
}
