//! `POST /uploads/{id}`: the multipart upload finish.
//!
//! The only route outside `/rpc`, because its argument is a byte stream
//! rather than a serde value. It answers with the same JSON-RPC response
//! object, `id` being `null`.

use std::sync::Arc;

use axum::{
    extract::{
        DefaultBodyLimit, FromRequest, Multipart, Path, Request, State, rejection::PathRejection,
    },
    http::{HeaderMap, HeaderValue, header},
    response::Response,
    routing,
};
use futures::stream;
use tokio::{sync::mpsc, task::JoinHandle};

use viendesu_core::service::{CallStep as _, uploads::Uploads as _};
use viendesu_protocol::{
    errors::Aux,
    requests::uploads::finish,
    types::upload,
    uploads::{AbortReason, Chunk, UploadStream},
};

use crate::{format::Format, server::Types};

use super::{
    code,
    envelope::{self, Id},
    extract,
    methods::Call,
    reply, session,
};

pub fn router<T: Types>(service: Arc<T::Service>) -> axum::Router {
    axum::Router::new().route(
        "/uploads/{id}",
        routing::post(finish::<T>)
            .layer(DefaultBodyLimit::disable())
            .with_state(service),
    )
}

async fn finish<T: Types>(
    State(service): State<Arc<T::Service>>,
    id: Result<Path<upload::Id>, PathRejection>,
    headers: HeaderMap,
    request: Request,
) -> Response {
    let format = extract::response_format(&headers, Format::Json);
    let body = outcome::<T>(&service, id, &headers, format, request).await;

    let mut response = reply(format, body);
    // The body may stay unread (early rejection, service error), which spoils the connection.
    response
        .headers_mut()
        .insert(header::CONNECTION, HeaderValue::from_static("close"));
    response
}

async fn outcome<T: Types>(
    service: &T::Service,
    id: Result<Path<upload::Id>, PathRejection>,
    headers: &HeaderMap,
    format: Format,
    request: Request,
) -> Vec<u8> {
    let call = Call {
        format,
        id: &Id::Null,
        params: envelope::Params::Absent,
    };

    let id = match id {
        Ok(Path(id)) => id,
        Err(rejection) => {
            let message = format!("invalid params: upload id: {rejection}");
            return call.fail(code::INVALID_PARAMS, &message);
        }
    };

    let mut session = match session::<T>(service, headers, &call).await {
        Ok(session) => session,
        Err(body) => return body,
    };

    let result = session
        .uploads()
        .finish()
        .call(finish::Args {
            id,
            stream: multipart_stream(request),
        })
        .await;

    call.finish(result)
}

/// Streams the single multipart field; `Multipart` borrows the request,
/// so a task owns it and forwards chunks.
fn multipart_stream(request: Request) -> UploadStream<'static> {
    struct Reader {
        handle: JoinHandle<()>,
        rx: mpsc::Receiver<Chunk>,
    }

    impl Drop for Reader {
        fn drop(&mut self) {
            self.handle.abort();
        }
    }

    let (tx, rx) = mpsc::channel(1);
    let handle = tokio::spawn(async move {
        if let Err(e) = forward(request, &tx).await {
            _ = tx
                .send(Chunk::Aborted(AbortReason::Other(e.to_string())))
                .await;
        }
    });

    let chunks = stream::unfold(Reader { handle, rx }, async |mut reader| {
        let chunk = reader.rx.recv().await?;
        Some((chunk, reader))
    });
    UploadStream::unknown_size(Box::pin(chunks))
}

async fn forward(request: Request, tx: &mpsc::Sender<Chunk>) -> Result<(), Aux> {
    let mut multipart = Multipart::from_request(request, &())
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
