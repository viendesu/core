use crate::{
    format::Format,
    server::{
        request::{ServerRequest, extract},
        response,
    },
};

use axum::{
    RequestExt,
    extract::Request as AxumRequest,
    http::{Method, request::Parts},
    response::Response as AxumResponse,
};

use futures::StreamExt;
use serde::Deserialize;

use viendesu_core::{
    errors::{Aux, AuxResult},
    types::session,
};

#[non_exhaustive]
pub struct Context<R: ServerRequest> {
    pub request: R,
    pub token: Option<session::Token>,
    pub parts: Option<Parts>,
    pub response_format: Format,
}

impl<R: ServerRequest> Context<R> {
    pub async fn path<P>(&mut self) -> AuxResult<P>
    where
        P: for<'de> Deserialize<'de> + Send + 'static,
    {
        extract::path(self.parts.as_mut().unwrap()).await
    }

    pub fn query<'this, T: serde::Deserialize<'this>>(&'this self) -> AuxResult<T> {
        extract::query(self.parts.as_ref().unwrap())
    }
}

pub async fn load_args<R: ServerRequest>(req: AxumRequest) -> Result<Context<R>, AxumResponse>
where
    R: for<'de> serde::Deserialize<'de>,
{
    let (parts, body) = req.with_limited_body().into_parts();

    let response_format =
        extract::response_format(&parts).map_err(|e| response::err(Format::default(), e))?;

    let request: R = if parts.method == Method::GET {
        let query = parts.uri.query().unwrap_or("");
        serde_urlencoded::from_str(query)
            .map_err(|e| Aux::Deserialization(format!("failed to decode query: {e}")))
            .map_err(|e| response::err(response_format, e))?
    } else {
        let request_format =
            extract::request_format(&parts).map_err(|e| response::err(response_format, e))?;
        let content_length =
            extract::content_length(&parts).map_err(|e| response::err(response_format, e))?;
        let mut data_stream = body.into_data_stream();
        let mut buffer = Vec::with_capacity(content_length);

        while let Some(frame) = data_stream.next().await {
            let frame = match frame {
                Ok(f) => f,
                Err(e) => {
                    return Err(response::err(
                        response_format,
                        Aux::Deserialization(format!("failed to read body frame: {e}")),
                    ));
                }
            };

            buffer.extend_from_slice(&frame);
        }

        request_format
            .load(&buffer)
            .map_err(|e| Aux::Deserialization(format!("failed to deserialize body: {e:#}")))
            .map_err(|e| response::err(response_format, e))?
    };

    Ok(Context {
        request,
        token: extract::session_token(&parts).map_err(|e| response::err(response_format, e))?,
        parts: Some(parts),
        response_format,
    })
}
