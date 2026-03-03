use super::*;

use crate::{
    requests::uploads::{Abort, Finish, ListPending, Start},
    server::{
        handler::{Handler, delete, get, post},
        request::extract,
        response,
    },
};

use futures::stream;
use tokio::{sync::mpsc, task::JoinHandle};

use viendesu_core::{
    errors::Aux,
    requests::uploads::{abort, finish, list_pending, start},
    service::uploads::Uploads as _,
    types::upload,
    uploads::{AbortReason, Chunk, UploadStream},
};

use axum::{
    extract::{FromRequest, Multipart, Request as AxumRequest},
    response::Response as AxumResponse,
};

async fn populate_stream(req: AxumRequest, tx: &mpsc::Sender<Chunk>) -> Result<(), Aux> {
    let mut multipart = Multipart::from_request(req, &())
        .await
        .map_err(|e| Aux::Deserialization(format!("failed to load multipart request: {e}")))?;

    let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| Aux::Deserialization(format!("failed to read multipart field: {e}")))?
    else {
        return Err(Aux::Deserialization("expected one multipart field".into()));
    };

    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|e| Aux::Deserialization(format!("failed to read multipart chunk: {e}")))?
    {
        if tx.send(Chunk::Data(chunk)).await.is_err() {
            break;
        }
    }

    Ok(())
}

async fn load_upload_context(request: AxumRequest) -> Result<Ctx<Finish>, AxumResponse> {
    struct State {
        handle: JoinHandle<()>,
        rx: mpsc::Receiver<Chunk>,
    }

    impl Drop for State {
        fn drop(&mut self) {
            // FUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUCK.
            // Most likely it's actually unneeded, BUT
            // let's not leave dangling tasks for whatever reason.
            self.handle.abort();
        }
    }

    let (mut parts, body) = request.into_parts();
    let response_format =
        extract::response_format(&parts).map_err(|e| response::err(Default::default(), e))?;
    let token = extract::session_token(&parts).map_err(|e| response::err(response_format, e))?;
    let id: upload::Id = extract::path(&mut parts)
        .await
        .map_err(|e| response::err(response_format, e))?;

    // This fucking sucks to the extent I can't express with words, the way its done
    // is horrible, the reason behind this is even more, I wish authors of axum very pleasant
    // requirement of self-referrential types.
    //
    //
    // Ногами.
    let (tx, rx) = mpsc::channel(1);
    let handle = tokio::spawn(async move {
        let tx = tx;
        if let Err(e) = populate_stream(AxumRequest::from_parts(parts, body), &tx).await {
            _ = tx
                .send(Chunk::Aborted(AbortReason::Other(format!("{e}"))))
                .await;
        }
    });
    let stream = stream::unfold(State { rx, handle }, async move |mut state| {
        if let Some(c) = state.rx.recv().await {
            Some((c, state))
        } else {
            None
        }
    });

    Ok(Ctx {
        token,
        request: Finish {
            id,
            stream: UploadStream::unknown_size(Box::pin(stream)),
        },
        parts: None,
        response_format,
    })
}

pub fn make<T: Types>(router: RouterScope<T>) -> RouterScope<T> {
    router
        .route(
            "/",
            get(async |mut session: SessionOf<T>, ctx: Ctx<ListPending>| {
                let ListPending {} = ctx.request;
                session
                    .uploads()
                    .list_pending()
                    .call(list_pending::Args {})
                    .await
            }),
        )
        .route(
            "/{id}",
            Handler::post(load_upload_context).exec(
                async |mut session: SessionOf<T>, ctx: Ctx<Finish>| {
                    let Finish { id, stream } = ctx.request;

                    session
                        .uploads()
                        .finish()
                        .call(finish::Args { id, stream })
                        .await
                },
            ),
        )
        .route(
            "/{id}",
            delete(async |mut session: SessionOf<T>, mut ctx: Ctx<Abort>| {
                let upload: upload::Id = ctx.path().await?;
                let Abort {} = ctx.request;

                session.uploads().abort().call(abort::Args { upload }).await
            }),
        )
        .route(
            "/",
            post(async |mut session: SessionOf<T>, ctx: Ctx<Start>| {
                let Start {
                    file_name,
                    hash,
                    class,
                    size,
                } = ctx.request;
                session
                    .uploads()
                    .start()
                    .call(start::Args {
                        name: file_name,
                        hash,
                        class,
                        size,
                    })
                    .await
            }),
        )
}
