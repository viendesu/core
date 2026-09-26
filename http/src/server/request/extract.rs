use axum::{
    extract::{FromRequestParts, Path},
    http::{HeaderMap, HeaderValue, request::Parts},
};

use viendesu_protocol::{
    errors::{Aux, AuxResult},
    types::session,
};

use crate::format::Format;

pub fn session_token(headers: &HeaderMap) -> AuxResult<Option<session::Token>> {
    let Some(val) = str_header(headers, "authorization")? else {
        return Ok(None);
    };
    let Some((scheme, rest)) = val.split_once(' ') else {
        return Err(Aux::Deserialization(
            "invalid Authorization header format, expected `<scheme> <rest>`".into(),
        ));
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

pub fn str_header<'h>(headers: &'h HeaderMap, header: &str) -> AuxResult<Option<&'h str>> {
    let Some(value) = raw_header(headers, header) else {
        return Ok(None);
    };
    let value = value.to_str().map_err(|e| {
        Aux::Deserialization(format!(
            "failed to decode UTF-8 content of header {header:?}: {e}"
        ))
    })?;

    Ok(Some(value))
}

pub fn raw_header<'h>(headers: &'h HeaderMap, header: &str) -> Option<&'h HeaderValue> {
    headers.get(header)
}

pub fn request_format(headers: &HeaderMap) -> AuxResult<Format> {
    let Some(raw) = str_header(headers, "content-type")? else {
        return Err(Aux::Deserialization(
            "`Content-Type` header is required".into(),
        ));
    };
    Format::from_mime_type(raw)
        .map_err(|e| Aux::Deserialization(format!("failed to parse `Content-Type` header: {e}")))
}

/// Response format requested by `Accept`, `fallback` when it is absent.
pub fn response_format(headers: &HeaderMap, fallback: Format) -> AuxResult<Format> {
    let Some(raw) = str_header(headers, "accept")? else {
        return Ok(fallback);
    };

    Format::negotiate(raw, fallback).ok_or_else(|| {
        Aux::Deserialization(format!(
            "none of the `Accept` media types is supported: {raw}"
        ))
    })
}

pub fn content_length(headers: &HeaderMap) -> AuxResult<usize> {
    let Some(raw) = str_header(headers, "content-length")? else {
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
