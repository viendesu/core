use axum::http::{HeaderMap, HeaderValue};

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
    let Some(value) = headers.get(header).map(HeaderValue::to_str) else {
        return Ok(None);
    };
    let value = value.map_err(|e| {
        Aux::Deserialization(format!(
            "failed to decode UTF-8 content of header {header:?}: {e}"
        ))
    })?;

    Ok(Some(value))
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

/// Codec of the reply: what `Accept` prefers, `fallback` when it is absent,
/// unreadable or names nothing supported.
pub fn response_format(headers: &HeaderMap, fallback: Format) -> Format {
    match str_header(headers, "accept") {
        Ok(Some(accept)) => Format::negotiate(accept, fallback).unwrap_or(fallback),
        _ => fallback,
    }
}
