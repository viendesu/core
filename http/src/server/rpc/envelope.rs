//! JSON-RPC 2.0 request/response objects in both codecs.

use std::{borrow::Cow, fmt};

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, DeserializeOwned},
};
use serde_json::value::RawValue;

use crate::format::{DumpParams, Format, msgpack_value_len};

use super::code;

/// Request id, echoed back verbatim.
#[derive(Debug, Clone, PartialEq)]
pub enum Id {
    Null,
    Int(i64),
    Uint(u64),
    Str(String),
}

impl Serialize for Id {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_none(),
            Self::Int(i) => serializer.serialize_i64(*i),
            Self::Uint(u) => serializer.serialize_u64(*u),
            Self::Str(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl de::Visitor<'_> for Visitor {
            type Value = Id;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string, an integer or null")
            }

            fn visit_unit<E>(self) -> Result<Id, E> {
                Ok(Id::Null)
            }

            fn visit_none<E>(self) -> Result<Id, E> {
                Ok(Id::Null)
            }

            fn visit_i64<E>(self, v: i64) -> Result<Id, E> {
                Ok(Id::Int(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Id, E> {
                Ok(Id::Uint(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Id, E> {
                Ok(Id::Str(v.to_owned()))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Parameters of a call, still in the wire codec.
pub enum Params<'a> {
    Absent,
    Json(&'a RawValue),
    Msgpack(&'a [u8]),
    /// Positional or scalar params; every method takes a named-field object.
    NotAnObject,
}

impl Params<'_> {
    /// Decodes through [`Format::load`], the same path REST bodies take.
    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, String> {
        let result = match *self {
            Self::Absent => Format::Json.load(b"{}"),
            Self::Json(raw) => Format::Json.load(raw.get().as_bytes()),
            Self::Msgpack(bytes) => Format::Msgpack.load(bytes),
            Self::NotAnObject => return Err("invalid params: params must be an object".to_owned()),
        };

        result.map_err(|e| format!("invalid params: {e:#}"))
    }
}

pub struct Request<'a> {
    /// `None` for notifications.
    pub id: Option<Id>,
    pub method: Cow<'a, str>,
    pub params: Params<'a>,
}

/// Why a body is not a request; answered with `id: null`.
pub struct Rejection {
    pub code: i32,
    pub message: String,
}

impl Rejection {
    fn parse(message: impl fmt::Display) -> Self {
        Self {
            code: code::PARSE_ERROR,
            message: format!("parse error: {message}"),
        }
    }

    fn invalid(message: impl fmt::Display) -> Self {
        Self {
            code: code::INVALID_REQUEST,
            message: format!("invalid request: {message}"),
        }
    }
}

const BATCH: &str = "batch requests are not supported";

pub fn parse(format: Format, body: &[u8]) -> Result<Request<'_>, Rejection> {
    match format {
        Format::Json => parse_json(body),
        Format::Msgpack => parse_msgpack(body),
    }
}

fn check_version(version: &str) -> Result<(), Rejection> {
    if version == "2.0" {
        Ok(())
    } else {
        Err(Rejection::invalid(format_args!(
            "unsupported jsonrpc version {version:?}"
        )))
    }
}

fn parse_json(body: &[u8]) -> Result<Request<'_>, Rejection> {
    #[derive(Deserialize)]
    struct Envelope<'a> {
        #[serde(borrow)]
        jsonrpc: Cow<'a, str>,
        #[serde(default, deserialize_with = "present")]
        id: Option<Id>,
        #[serde(borrow)]
        method: Cow<'a, str>,
        #[serde(default, borrow)]
        params: Option<&'a RawValue>,
    }

    fn present<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Id>, D::Error> {
        Id::deserialize(deserializer).map(Some)
    }

    let raw: &RawValue = serde_json::from_slice(body).map_err(Rejection::parse)?;
    match raw.get().as_bytes().first() {
        Some(b'{') => {}
        Some(b'[') => return Err(Rejection::invalid(BATCH)),
        _ => return Err(Rejection::invalid("request must be an object")),
    }

    let envelope: Envelope<'_> = serde_json::from_str(raw.get()).map_err(Rejection::invalid)?;
    check_version(&envelope.jsonrpc)?;

    let params = match envelope.params {
        None => Params::Absent,
        Some(raw) => match raw.get().as_bytes().first() {
            Some(b'{') => Params::Json(raw),
            Some(b'n') => Params::Absent,
            _ => Params::NotAnObject,
        },
    };

    Ok(Request {
        id: envelope.id,
        method: envelope.method,
        params,
    })
}

fn parse_msgpack(body: &[u8]) -> Result<Request<'_>, Rejection> {
    use rmp::{Marker, decode::DecodeStringError};

    fn string(raw: &[u8]) -> Option<&str> {
        match rmp::decode::read_str_from_slice(raw) {
            Ok((s, [])) => Some(s),
            _ => None,
        }
    }

    let len = msgpack_value_len(body).map_err(|e| {
        if e.is_malformed() {
            Rejection::parse(e)
        } else {
            Rejection::invalid(e)
        }
    })?;
    if len != body.len() {
        return Err(Rejection::parse("trailing data after the request"));
    }
    match Marker::from_u8(body[0]) {
        Marker::FixMap(_) | Marker::Map16 | Marker::Map32 => {}
        Marker::FixArray(_) | Marker::Array16 | Marker::Array32 => {
            return Err(Rejection::invalid(BATCH));
        }
        _ => return Err(Rejection::invalid("request must be a map")),
    }

    let mut rest = body;
    let entries = rmp::decode::read_map_len(&mut rest).map_err(Rejection::parse)?;

    let mut version = None;
    let mut id = None;
    let mut method = None;
    let mut params = Params::Absent;
    let mut seen = [false; 4];

    for _ in 0..entries {
        let (key, tail) = rmp::decode::read_str_from_slice(rest).map_err(|e| match e {
            DecodeStringError::TypeMismatch(_) => Rejection::invalid("map keys must be strings"),
            e => Rejection::parse(e),
        })?;
        let len = msgpack_value_len(tail).map_err(Rejection::parse)?;
        let (raw, tail) = tail.split_at(len);
        rest = tail;

        if let Some(member) = ["jsonrpc", "id", "method", "params"]
            .iter()
            .position(|m| *m == key)
        {
            if std::mem::replace(&mut seen[member], true) {
                return Err(Rejection::invalid(format_args!("duplicate member {key:?}")));
            }
        }

        match key {
            "jsonrpc" => version = string(raw),
            "id" => {
                let parsed = rmp_serde::from_slice::<Id>(raw)
                    .map_err(|_| Rejection::invalid("id must be a string, an integer or nil"))?;
                id = Some(parsed);
            }
            "method" => {
                let name =
                    string(raw).ok_or_else(|| Rejection::invalid("method must be a string"))?;
                method = Some(Cow::Borrowed(name));
            }
            "params" => {
                params = match Marker::from_u8(raw[0]) {
                    Marker::FixMap(_) | Marker::Map16 | Marker::Map32 => Params::Msgpack(raw),
                    Marker::Null => Params::Absent,
                    _ => Params::NotAnObject,
                };
            }
            _ => {}
        }
    }

    check_version(version.unwrap_or_default())?;
    let method = method.ok_or_else(|| Rejection::invalid("missing method"))?;

    Ok(Request { id, method, params })
}

#[derive(Serialize)]
struct Success<'a, T: ?Sized> {
    jsonrpc: &'static str,
    id: &'a Id,
    result: &'a T,
}

#[derive(Serialize)]
struct Failure<'a, D: ?Sized> {
    jsonrpc: &'static str,
    id: &'a Id,
    error: ErrorObject<'a, D>,
}

#[derive(Serialize)]
struct ErrorObject<'a, D: ?Sized> {
    code: i32,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<&'a D>,
}

fn dump<T: Serialize + ?Sized>(format: Format, what: &T) -> Vec<u8> {
    let mut body = Vec::with_capacity(128);
    format.dump(DumpParams::default(), what, &mut body);
    body
}

pub fn success<T: Serialize + ?Sized>(format: Format, id: &Id, result: &T) -> Vec<u8> {
    dump(
        format,
        &Success {
            jsonrpc: "2.0",
            id,
            result,
        },
    )
}

pub fn failure<D: Serialize + ?Sized>(
    format: Format,
    id: &Id,
    code: i32,
    message: &str,
    data: Option<&D>,
) -> Vec<u8> {
    dump(
        format,
        &Failure {
            jsonrpc: "2.0",
            id,
            error: ErrorObject {
                code,
                message,
                data,
            },
        },
    )
}

pub fn failure_plain(format: Format, id: &Id, code: i32, message: &str) -> Vec<u8> {
    failure::<()>(format, id, code, message, None)
}
