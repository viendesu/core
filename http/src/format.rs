use eva::data;
#[cfg(feature = "format-serde")]
use eva::error::ShitHappens;
#[cfg(feature = "format-serde")]
use eyre::Context;

#[cfg(feature = "format-serde")]
mod json_model;

#[data(copy, display("got unsupported mime type"), error)]
pub struct UnknownMimeType;

/// Wire codec.
///
/// MessagePack carries exactly the JSON shapes — values are encoded through
/// the JSON data model and decoded human-readable. That is a protocol
/// invariant: the untagged protocol enums decode through serde's buffered
/// content, which is always human-readable.
#[data(copy, display("{}", self.mime_type()))]
#[derive(Default)]
pub enum Format {
    #[default]
    Json,
    Msgpack,
}

impl Format {
    /// Parses a `Content-Type` value; parameters such as `charset` are ignored.
    pub fn from_mime_type(mime: &str) -> Result<Self, UnknownMimeType> {
        Self::from_essence(essence(mime)).ok_or(UnknownMimeType)
    }

    /// Picks the most preferred supported format of an `Accept` value.
    ///
    /// The most specific matching range sets a format's quality, so
    /// `application/msgpack;q=0, */*` excludes MessagePack. Ties go to
    /// `fallback` (the request codec), then to the explicitly named format, so
    /// a MessagePack client with a generic `application/json, */*` default still
    /// gets MessagePack; `None` if nothing is acceptable.
    pub fn negotiate(accept: &str, fallback: Self) -> Option<Self> {
        const FORMATS: [Format; 2] = [Format::Json, Format::Msgpack];
        // (specificity, quality) of the most specific range matching each format.
        let mut matched: [Option<(u8, f32)>; 2] = [None; 2];

        for range in accept.split(',') {
            let mut parts = range.split(';');
            let essence = parts.next().unwrap_or_default().trim();
            let quality = parts
                .filter_map(|param| {
                    let (name, value) = param.split_once('=')?;
                    name.trim()
                        .eq_ignore_ascii_case("q")
                        .then(|| value.trim().parse::<f32>().ok())
                        .flatten()
                })
                .next()
                .unwrap_or(1.0);
            let quality = if quality.is_nan() { 0.0 } else { quality };

            let (specificity, only) = if essence == "*/*" {
                (0, None)
            } else if essence.eq_ignore_ascii_case("application/*") {
                (1, None)
            } else if let Some(format) = Self::from_essence(essence) {
                (2, Some(format))
            } else {
                continue;
            };

            for (slot, format) in matched.iter_mut().zip(FORMATS) {
                if only.is_some_and(|only| only != format) {
                    continue;
                }
                *slot = match *slot {
                    Some((s, q)) if s > specificity => Some((s, q)),
                    Some((s, q)) if s == specificity => Some((s, q.max(quality))),
                    _ => Some((specificity, quality)),
                };
            }
        }

        matched
            .into_iter()
            .zip(FORMATS)
            .filter_map(|(slot, format)| {
                let (specificity, quality) = slot?;
                (quality > 0.0).then_some(((quality, format == fallback, specificity), format))
            })
            .max_by(|(a, _), (b, _)| a.partial_cmp(b).expect("qualities are not NaN"))
            .map(|(_, format)| format)
    }

    pub const fn mime_type(self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::Msgpack => "application/msgpack",
        }
    }

    fn from_essence(essence: &str) -> Option<Self> {
        const MSGPACK: [&str; 3] = [
            "application/msgpack",
            "application/x-msgpack",
            "application/vnd.msgpack",
        ];

        if essence.eq_ignore_ascii_case("application/json") {
            Some(Self::Json)
        } else if MSGPACK.iter().any(|m| essence.eq_ignore_ascii_case(m)) {
            Some(Self::Msgpack)
        } else {
            None
        }
    }
}

fn essence(mime: &str) -> &str {
    mime.split(';').next().unwrap_or_default().trim()
}

#[cfg(feature = "format-serde")]
#[data]
#[derive(Default)]
pub struct DumpParams {
    pub pretty: bool,
}

#[cfg(feature = "format-serde")]
impl Format {
    pub fn dump<T: ?Sized>(self, params: DumpParams, what: &T, dst: &mut Vec<u8>)
    where
        T: serde::Serialize,
    {
        match self {
            Self::Json => if params.pretty {
                serde_json::to_writer_pretty(dst, what)
            } else {
                serde_json::to_writer(dst, what)
            }
            .shit_happens(),
            Self::Msgpack => {
                let mut serializer = rmp_serde::Serializer::new(dst)
                    .with_struct_map()
                    .with_human_readable();
                serde::Serialize::serialize(&json_model::JsonModel(what), &mut serializer)
                    .shit_happens();
            }
        }
    }

    pub fn load<'de, T>(&self, buf: &'de [u8]) -> eyre::Result<T>
    where
        T: serde::Deserialize<'de>,
    {
        match self {
            Self::Json => {
                let de = &mut serde_json::Deserializer::from_slice(buf);
                let value = serde_path_to_error::deserialize(&mut *de)
                    .wrap_err("failed to deserialize JSON")?;
                de.end().wrap_err("trailing data after JSON")?;
                Ok(value)
            }
            Self::Msgpack => {
                let len = msgpack_value_len(buf).wrap_err("unacceptable MessagePack")?;
                if len != buf.len() {
                    eyre::bail!("trailing data after MessagePack");
                }

                // Borrowing reader: eva's `Date` only accepts borrowed strings.
                let mut de = rmp_serde::Deserializer::from_read_ref(buf).with_human_readable();
                de.set_max_depth(MAX_DEPTH);
                serde_path_to_error::deserialize(&mut de)
                    .wrap_err("failed to deserialize MessagePack")
            }
        }
    }
}

/// Nesting limit of MessagePack documents, as serde_json's default.
#[cfg(feature = "format-serde")]
const MAX_DEPTH: usize = 128;

#[cfg(feature = "format-serde")]
#[data(copy, error, display(doc))]
pub enum MsgpackError {
    /// malformed MessagePack
    Malformed,
    /// MessagePack value has no JSON counterpart
    NotJson,
}

/// Length in bytes of the MessagePack value at the start of `buf`, which must
/// be JSON-shaped: string map keys, no binary or extension values.
///
/// Serde would otherwise bind integer keys to struct fields by declaration index.
#[cfg(feature = "format-serde")]
pub fn msgpack_value_len(buf: &[u8]) -> Result<usize, MsgpackError> {
    use MsgpackError::*;
    use rmp::{
        Marker::{self, *},
        decode,
    };

    fn peek(rd: &[u8]) -> Result<Marker, MsgpackError> {
        rd.first().map(|&b| Marker::from_u8(b)).ok_or(Malformed)
    }

    fn advance(rd: &mut &[u8], n: usize) -> Result<(), MsgpackError> {
        *rd = rd.get(n..).ok_or(Malformed)?;
        Ok(())
    }

    /// Skips one value; `depth` bounds the recursion.
    fn skip(rd: &mut &[u8], depth: usize) -> Result<(), MsgpackError> {
        let (items, is_map) = match peek(rd)? {
            FixArray(_) | Array16 | Array32 => {
                (decode::read_array_len(rd).map_err(|_| Malformed)?, false)
            }
            FixMap(_) | Map16 | Map32 => (decode::read_map_len(rd).map_err(|_| Malformed)?, true),
            FixStr(_) | Str8 | Str16 | Str32 => {
                let len = decode::read_str_len(rd).map_err(|_| Malformed)?;
                return advance(rd, len as usize);
            }
            FixPos(_) | FixNeg(_) | Null | True | False => return advance(rd, 1),
            U8 | I8 => return advance(rd, 2),
            U16 | I16 => return advance(rd, 3),
            U32 | I32 | F32 => return advance(rd, 5),
            U64 | I64 | F64 => return advance(rd, 9),
            Reserved => return Err(Malformed),
            _ => return Err(NotJson),
        };

        let depth = depth.checked_sub(1).ok_or(Malformed)?;
        for _ in 0..items {
            if is_map {
                if !matches!(peek(rd)?, FixStr(_) | Str8 | Str16 | Str32) {
                    return Err(NotJson);
                }
                skip(rd, depth)?;
            }
            skip(rd, depth)?;
        }
        Ok(())
    }

    let mut rest = buf;
    skip(&mut rest, MAX_DEPTH)?;
    Ok(buf.len() - rest.len())
}

#[cfg(test)]
mod tests {
    use super::Format::{self, *};

    #[test]
    fn content_type_ignores_parameters() {
        assert_eq!(
            Format::from_mime_type("application/json; charset=utf-8").unwrap(),
            Json
        );
        assert_eq!(
            Format::from_mime_type("Application/MsgPack").unwrap(),
            Msgpack
        );
        assert_eq!(
            Format::from_mime_type("application/x-msgpack").unwrap(),
            Msgpack
        );
        assert!(Format::from_mime_type("*/*").is_err());
        assert!(Format::from_mime_type("text/plain").is_err());
    }

    #[test]
    fn accept_negotiation() {
        let n = |accept| Format::negotiate(accept, Json);

        assert_eq!(n("application/json, text/plain, */*"), Some(Json));
        assert_eq!(n("application/msgpack"), Some(Msgpack));
        assert_eq!(
            n("application/json;q=0.5, application/msgpack"),
            Some(Msgpack)
        );
        assert_eq!(n("*/*, application/msgpack"), Some(Json));
        assert_eq!(n("*/*;q=0.9, application/msgpack"), Some(Msgpack));
        assert_eq!(n("application/json, application/msgpack"), Some(Json));
        assert_eq!(
            Format::negotiate("application/json, text/plain, */*", Msgpack),
            Some(Msgpack)
        );
        assert_eq!(n("application/msgpack;q=0, */*;q=0.1"), Some(Json));
        assert_eq!(Format::negotiate("*/*", Msgpack), Some(Msgpack));
        assert_eq!(
            Format::negotiate("application/msgpack;q=0, */*", Msgpack),
            Some(Json)
        );
        assert_eq!(n("application/json;q=0, */*"), Some(Msgpack));
        assert_eq!(
            n("*/*, application/json;q=0, application/msgpack;q=0"),
            None
        );
        assert_eq!(
            n("application/json;q=0.5, application/json;q=0.9, application/msgpack;q=0.7"),
            Some(Json)
        );
        assert_eq!(n("text/html"), None);
    }
}
