use super::*;

use crate::requests::authors::Get;

use viendesu_core::service::authors::Authors;
use viendesu_protocol::{
    requests::authors::{create, get, search, update},
    types::author,
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
