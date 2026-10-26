//! JSON-RPC 2.0 request/response objects in both codecs.

use std::{borrow::Cow, fmt};

use eva::str::CompactString;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, DeserializeOwned},
};
use serde_json::value::RawValue;

use crate::format::{DumpParams, Format, MsgpackError, load_validated_msgpack, msgpack_value_len};

use super::code;

/// Request id, echoed back verbatim.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Id {
    Null,
    Int(i64),
    Uint(u64),
    Str(CompactString),
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
                Ok(Id::Str(v.into()))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Parameters of a call, still in the wire codec.
///
/// Members are unordered, so `params` may precede `method` and is decoded once
/// the method is known; being borrowed, it costs only re-scans of its bytes.
pub enum Params<'a> {
    Absent,
    Json(&'a RawValue),
    Msgpack(&'a [u8]),
    /// Positional or scalar params; every method takes a named-field object.
    NotAnObject,
}

impl Params<'_> {
    /// Decodes params without revalidating the enclosing MessagePack document.
    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, String> {
        let result = match *self {
            Self::Absent => Format::Json.load(b"{}"),
            Self::Json(raw) => Format::Json.load(raw.get().as_bytes()),
            Self::Msgpack(bytes) => load_validated_msgpack(bytes),
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

    pub fn answer(&self, format: Format) -> Vec<u8> {
        failure_plain(format, &Id::Null, self.code, &self.message)
    }

    pub fn answer_into(&self, format: Format, body: &mut Vec<u8>) {
        failure_into::<()>(format, &Id::Null, self.code, &self.message, None, body);
    }
}

/// A request body: one request, or a batch whose elements are rejected one by one.
pub enum Message<'a> {
    Single(Request<'a>),
    Batch(Vec<Result<Request<'a>, Rejection>>),
}

/// A batch longer than `max_batch` is rejected without dispatching any calls.
pub fn parse(format: Format, body: &[u8], max_batch: usize) -> Result<Message<'_>, Rejection> {
    match format {
        Format::Json => parse_json(body, max_batch),
        Format::Msgpack => parse_msgpack(body, max_batch),
    }
}

fn check_batch_len(len: usize, max_batch: usize) -> Result<(), Rejection> {
    match len {
        0 => Err(Rejection::invalid("empty batch")),
        len if len > max_batch => Err(Rejection::invalid(format_args!(
            "batch of {len} requests exceeds the limit of {max_batch}"
        ))),
        _ => Ok(()),
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

fn parse_json(body: &[u8], max_batch: usize) -> Result<Message<'_>, Rejection> {
    // Unknown envelope fields are skipped by serde, including their strings.
    // Validate UTF-8 once so malformed bytes in those fields are still rejected.
    let body = std::str::from_utf8(body).map_err(Rejection::parse)?;
    if json_start(body) != Some(b'[') {
        return json_request(body)
            .map(Message::Single)
            .or_else(|rejection| {
                // An envelope error can stop before malformed JSON later in the body.
                // Validate only rejected requests to preserve parse-error precedence.
                serde_json::from_str::<&RawValue>(body).map_err(Rejection::parse)?;
                Err(rejection)
            });
    }

    struct Batch(usize);

    impl<'de> de::Visitor<'de> for Batch {
        type Value = Result<Message<'de>, Rejection>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an array of requests")
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut requests = Vec::new();
            let mut count = 0;
            while let Some(raw) = seq.next_element::<&RawValue>()? {
                count += 1;
                if count <= self.0 {
                    requests.push(json_request(raw.get()));
                }
            }
            Ok(check_batch_len(count, self.0).map(|()| Message::Batch(requests)))
        }
    }

    let mut de = serde_json::Deserializer::from_str(body);
    let message = (&mut de)
        .deserialize_seq(Batch(max_batch))
        .map_err(Rejection::parse)?;
    de.end().map_err(Rejection::parse)?;
    message
}

fn json_start(body: &str) -> Option<u8> {
    body.bytes().find(|b| !b.is_ascii_whitespace())
}

fn json_request(body: &str) -> Result<Request<'_>, Rejection> {
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

    if json_start(body) != Some(b'{') {
        return Err(Rejection::invalid("request must be an object"));
    }

    let envelope: Envelope<'_> = serde_json::from_str(body).map_err(Rejection::invalid)?;
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

fn parse_msgpack(body: &[u8], max_batch: usize) -> Result<Message<'_>, Rejection> {
    use rmp::Marker;

    let len = msgpack_value_len(body).map_err(|e| match e {
        MsgpackError::Malformed => Rejection::parse(e),
        MsgpackError::NotJson => Rejection::invalid(e),
    })?;
    if len != body.len() {
        return Err(Rejection::parse("trailing data after the request"));
    }
    if !matches!(
        Marker::from_u8(body[0]),
        Marker::FixArray(_) | Marker::Array16 | Marker::Array32
    ) {
        return msgpack_request(body).map(Message::Single);
    }

    let mut rest = body;
    let count = rmp::decode::read_array_len(&mut rest).map_err(Rejection::parse)?;
    check_batch_len(count as usize, max_batch)?;

    let requests = (0..count)
        .map(|_| {
            // Cannot fail: the whole body already passed `msgpack_value_len`.
            let len = msgpack_value_len(rest).unwrap_or(rest.len());
            let (raw, tail) = rest.split_at(len);
            rest = tail;
            msgpack_request(raw)
        })
        .collect();
    Ok(Message::Batch(requests))
}

/// `raw` is exactly one well-formed value.
fn msgpack_request(raw: &[u8]) -> Result<Request<'_>, Rejection> {
    use rmp::{Marker, decode::DecodeStringError};

    fn string(raw: &[u8]) -> Option<&str> {
        match rmp::decode::read_str_from_slice(raw) {
            Ok((s, [])) => Some(s),
            _ => None,
        }
    }

    if !matches!(
        Marker::from_u8(raw[0]),
        Marker::FixMap(_) | Marker::Map16 | Marker::Map32
    ) {
        return Err(Rejection::invalid("request must be a map"));
    }

    let mut rest = raw;
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

pub fn success_into<T: Serialize + ?Sized>(
    format: Format,
    id: &Id,
    result: &T,
    body: &mut Vec<u8>,
) {
    body.reserve(128);
    format.dump(
        DumpParams::default(),
        &Success {
            jsonrpc: "2.0",
            id,
            result,
        },
        body,
    )
}

pub fn failure_into<D: Serialize + ?Sized>(
    format: Format,
    id: &Id,
    code: i32,
    message: &str,
    data: Option<&D>,
    body: &mut Vec<u8>,
) {
    body.reserve(128);
    format.dump(
        DumpParams::default(),
        &Failure {
            jsonrpc: "2.0",
            id,
            error: ErrorObject {
                code,
                message,
                data,
            },
        },
        body,
    )
}

pub fn failure_plain(format: Format, id: &Id, code: i32, message: &str) -> Vec<u8> {
    let mut body = Vec::new();
    failure_into::<()>(format, id, code, message, None, &mut body);
    body
}
