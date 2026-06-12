use super::*;

pub fn genres<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    use viendesu_core::service::marks::Genres as _;
    use viendesu_protocol::requests::marks::list_genres;

    router.route(
        "/",
        get(
            async move |mut session: SessionOf<T>, ctx: Ctx<list_genres::Args>| {
                session.genres().list().call(ctx.request).await
            },
        ),
    )
}

pub fn badges<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    use crate::requests::marks::AddBadge;

    use viendesu_core::service::marks::Badges as _;
    use viendesu_protocol::requests::marks::{add_badge, list_badges};

    router
        .route(
            "/",
            get(
                async move |mut session: SessionOf<T>, ctx: Ctx<list_badges::Args>| {
                    session.badges().list().call(ctx.request).await
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
    use crate::requests::marks::AddTag;

    use viendesu_core::service::marks::Tags as _;
    use viendesu_protocol::requests::marks::{add_tag, list_tags};

    router
        .route(
            "/",
            get(
                async move |mut session: SessionOf<T>, ctx: Ctx<list_tags::Args>| {
                    session.tags().list().call(ctx.request).await
                },
            ),
        )
        .route(
            "/",
            post(async move |mut session: SessionOf<T>, ctx: Ctx<AddTag>| {
                let AddTag { text } = ctx.request;
                session.tags().add().call(add_tag::Args { tag: text }).await
            }),
        )
}
