use axum::{
    extract::{FromRequest, FromRequestParts, Path, Request},
    http::{HeaderValue, request::Parts},
};

use eva::bytes::Bytes;
use viendesu_core::{
    errors::{Aux, AuxResult},
    types::{captcha, session},
};

use crate::format::Format;

pub async fn read_body<R: serde::de::DeserializeOwned>(
    format: Format,
    request: Request,
) -> AuxResult<R> {
    let raw_body = read_raw_body(request).await?;
    format
        .load(&raw_body)
        .map_err(|e| Aux::Deserialization(format!("failed to deserialize body: {e:#}")))
}

pub fn raw_query(parts: &Parts) -> &str {
    parts.uri.query().unwrap_or("")
}

pub fn query<'de, T: serde::de::Deserialize<'de>>(parts: &'de Parts) -> AuxResult<T> {
    let raw = raw_query(parts);
    serde_urlencoded::from_str(raw)
        .map_err(|e| Aux::Deserialization(format!("failed to decode query string: {e}")))
}

pub async fn read_raw_body(request: Request) -> AuxResult<Bytes> {
    match Bytes::from_request(request, &()).await {
        Ok(r) => Ok(r),
        Err(e) => Err(Aux::Deserialization(format!("failed to read body: {e}"))),
    }
}

pub fn session_token(parts: &Parts) -> AuxResult<Option<session::Token>> {
    let Some(val) = str_header(parts, "authorization")? else {
        return Ok(None);
    };
    let Some((scheme, rest)) = val.split_once(' ') else {
        return Err(Aux::Deserialization(format!(
            "invalid Authorization header format, expected `<scheme> <rest>`"
        )));
    };

    match scheme {
        "Bearer" => rest
            .parse()
            .map(Some)
            .map_err(|e| Aux::Deserialization(format!("invalid session token: {e}"))),
        _ => Err(Aux::Deserialization(format!(
            "scheme {scheme:?} is not supported"
        ))),
    }
}

pub fn captcha(parts: &Parts) -> AuxResult<Option<captcha::Token>> {
    let Some(raw_token) = str_header(parts, "x-challenge")? else {
        return Ok(None);
    };

    match raw_token.parse() {
        Ok(r) => Ok(Some(r)),
        Err(e) => Err(Aux::Deserialization(format!(
            "invalid X-Challenge header: {e}"
        ))),
    }
}

pub fn str_header<'p>(parts: &'p Parts, header: &str) -> AuxResult<Option<&'p str>> {
    let Some(value) = raw_header(parts, header) else {
        return Ok(None);
    };
    let value = value.to_str().map_err(|e| {
        Aux::Deserialization(format!(
            "failed to decode UTF-8 content of header {header:?}: {e}"
        ))
    })?;

    Ok(Some(value))
}

pub fn raw_header<'p>(parts: &'p Parts, header: &str) -> Option<&'p HeaderValue> {
    parts.headers.get(header)
}

pub fn request_format(parts: &Parts) -> AuxResult<Format> {
    let Some(raw) = str_header(parts, "content-type")? else {
        return Err(Aux::Deserialization(
            "`Content-Type` header is required".into(),
        ));
    };
    Format::from_mime_type(raw)
        .map_err(|e| Aux::Deserialization(format!("failed to parse `Content-Type` header: {e}")))
}

pub fn response_format(parts: &Parts) -> AuxResult<Format> {
    let Some(raw) = str_header(parts, "accept")? else {
        return Ok(Format::Json);
    };

    Format::from_mime_type(raw)
        .map_err(|e| Aux::Deserialization(format!("failed to parse `Accept` header: {e}")))
}

pub fn content_length(parts: &Parts) -> AuxResult<usize> {
    let Some(raw) = str_header(parts, "content-length")? else {
        return Ok(0);
    };

    let content_length: usize = raw
        .parse()
        .map_err(|e| Aux::Deserialization(format!("failed to decode content length: {e}")))?;

    Ok(content_length)
}

pub async fn path<T>(parts: &mut Parts) -> AuxResult<T>
where
    T: serde::de::DeserializeOwned + Send + 'static,
{
    let response = Path::<T>::from_request_parts(parts, &()).await;
    match response {
        Ok(Path(r)) => Ok(r),
        Err(rej) => Err(Aux::Deserialization(format!("failed to parse path: {rej}"))),
    }
}
