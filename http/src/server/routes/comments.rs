use super::*;

use crate::requests::comments::{Delete, Edit, Like, Replies};

use viendesu_core::service::comments::Comments;
use viendesu_protocol::{
    requests::comments::{delete, edit, like, replies},
    types::comment,
};

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/{comment_id}/replies",
            get(async |mut session: SessionOf<T>, mut ctx: Ctx<Replies>| {
                let comment: comment::Id = ctx.path().await?;
                let Replies { after, limit } = ctx.request;

                session
                    .comments()
                    .replies()
                    .call(replies::Args {
                        comment,
                        after,
                        limit,
                    })
                    .await
            }),
        )
        .route(
            "/{comment_id}",
            patch(async |mut session: SessionOf<T>, mut ctx: Ctx<Edit>| {
                let comment: comment::Id = ctx.path().await?;
                let Edit { text } = ctx.request;

                session
                    .comments()
                    .edit()
                    .call(edit::Args { comment, text })
                    .await
            }),
        )
        .route(
            "/{comment_id}",
            delete(async |mut session: SessionOf<T>, mut ctx: Ctx<Delete>| {
                let comment: comment::Id = ctx.path().await?;
                let Delete {} = ctx.request;

                session
                    .comments()
                    .delete()
                    .call(delete::Args { comment })
                    .await
            }),
        )
        .route(
            "/{comment_id}/like",
            put(async |mut session: SessionOf<T>, mut ctx: Ctx<Like>| {
                let comment: comment::Id = ctx.path().await?;
                let Like { liked } = ctx.request;

                session
                    .comments()
                    .like()
                    .call(like::Args { comment, liked })
                    .await
            }),
        )
}
