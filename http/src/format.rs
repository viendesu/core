use eva::data;
#[cfg(feature = "format-serde")]
use eva::error::ShitHappens;
#[cfg(feature = "format-serde")]
use eyre::Context;

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
                // Through the JSON data model: rmp-serde alone writes unit structs as `[]`.
                let tree = serde_json::to_value(what).shit_happens();
                let mut serializer = rmp_serde::Serializer::new(dst).with_human_readable();
                serde::Serialize::serialize(&tree, &mut serializer).shit_happens();
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
    /// truncated MessagePack value
    Truncated,
    /// reserved MessagePack marker 0xc1
    Reserved,
    /// MessagePack nested deeper than 128 levels
    TooDeep,
    /// MessagePack map key is not a string
    NonStringKey,
    /// MessagePack binary or extension value has no JSON counterpart
    NotJson,
}

#[cfg(feature = "format-serde")]
impl MsgpackError {
    /// Malformed input, as opposed to well-formed input JSON cannot carry.
    pub const fn is_malformed(self) -> bool {
        matches!(self, Self::Truncated | Self::Reserved | Self::TooDeep)
    }
}

/// Length in bytes of the MessagePack value at the start of `buf`, which must
/// be JSON-shaped: string map keys, no binary or extension values.
///
/// Serde would otherwise bind integer keys to struct fields by declaration
/// index. Walks markers iteratively over a depth-bounded stack, so hostile
/// input costs neither memory nor native stack.
#[cfg(feature = "format-serde")]
pub fn msgpack_value_len(buf: &[u8]) -> Result<usize, MsgpackError> {
    use rmp::Marker::*;

    fn be(buf: &[u8], pos: &mut usize, width: usize) -> Result<u64, MsgpackError> {
        let bytes = buf.get(*pos..*pos + width).ok_or(MsgpackError::Truncated)?;
        *pos += width;
        Ok(bytes.iter().fold(0, |acc, &b| acc << 8 | u64::from(b)))
    }

    // Items left in each open container (a map counts keys and values) and
    // whether it is a map.
    let mut open: Vec<(u64, bool)> = Vec::new();
    let mut pos = 0;

    loop {
        let is_key = match open.last_mut() {
            Some((left, is_map)) => {
                let is_key = *is_map && *left % 2 == 0;
                *left -= 1;
                is_key
            }
            None => false,
        };

        let marker = rmp::Marker::from_u8(*buf.get(pos).ok_or(MsgpackError::Truncated)?);
        pos += 1;
        if is_key && !matches!(marker, FixStr(_) | Str8 | Str16 | Str32) {
            return Err(MsgpackError::NonStringKey);
        }

        let (payload, container) = match marker {
            FixPos(_) | FixNeg(_) | Null | True | False => (0, None),
            U8 | I8 => (1, None),
            U16 | I16 => (2, None),
            U32 | I32 | F32 => (4, None),
            U64 | I64 | F64 => (8, None),
            FixStr(len) => (u64::from(len), None),
            Str8 => (be(buf, &mut pos, 1)?, None),
            Str16 => (be(buf, &mut pos, 2)?, None),
            Str32 => (be(buf, &mut pos, 4)?, None),
            FixArray(len) => (0, Some((u64::from(len), false))),
            Array16 => (0, Some((be(buf, &mut pos, 2)?, false))),
            Array32 => (0, Some((be(buf, &mut pos, 4)?, false))),
            FixMap(len) => (0, Some((2 * u64::from(len), true))),
            Map16 => (0, Some((2 * be(buf, &mut pos, 2)?, true))),
            Map32 => (0, Some((2 * be(buf, &mut pos, 4)?, true))),
            Bin8 | Bin16 | Bin32 | FixExt1 | FixExt2 | FixExt4 | FixExt8 | FixExt16 | Ext8
            | Ext16 | Ext32 => return Err(MsgpackError::NotJson),
            Reserved => return Err(MsgpackError::Reserved),
        };

        let payload = usize::try_from(payload).map_err(|_| MsgpackError::Truncated)?;
        pos = pos
            .checked_add(payload)
            .filter(|&end| end <= buf.len())
            .ok_or(MsgpackError::Truncated)?;

        if let Some(container) = container.filter(|&(items, _)| items > 0) {
            if open.len() == MAX_DEPTH {
                return Err(MsgpackError::TooDeep);
            }
            open.push(container);
        }
        while open.last().is_some_and(|&(left, _)| left == 0) {
            open.pop();
        }
        if open.is_empty() {
            return Ok(pos);
        }
    }
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

    #[cfg(feature = "format-serde")]
    mod msgpack {
        use super::super::{MsgpackError, msgpack_value_len};

        fn encode(value: &serde_json::Value) -> Vec<u8> {
            rmp_serde::to_vec(value).unwrap()
        }

        #[test]
        fn value_lengths() {
            let values = [
                serde_json::json!(null),
                serde_json::json!(true),
                serde_json::json!(-1),
                serde_json::json!(u64::MAX),
                serde_json::json!(i64::MIN),
                serde_json::json!(1.5),
                serde_json::json!("x".repeat(40)),
                serde_json::json!("y".repeat(300)),
                serde_json::json!("z".repeat(70_000)),
                serde_json::json!([1, [2, [3, {"a": [null]}]], "s"]),
                serde_json::json!({"k": {"n": [1, 2, 3], "m": {}}, "e": []}),
                serde_json::json!((0..20).collect::<Vec<_>>()),
                serde_json::json!((0..70_000).map(|_| 0).collect::<Vec<_>>()),
                serde_json::json!(
                    (0..20)
                        .map(|i| (format!("k{i}"), serde_json::json!(i)))
                        .collect::<serde_json::Map<_, _>>()
                ),
            ];

            for value in values {
                let mut bytes = encode(&value);
                let len = bytes.len();
                assert_eq!(msgpack_value_len(&bytes), Ok(len), "{value}");

                bytes.extend_from_slice(&[0xc0, 0x01]);
                assert_eq!(msgpack_value_len(&bytes), Ok(len), "{value}");
            }
        }

        #[test]
        fn only_json_shapes() {
            let bin = [0xc4, 1, 7];
            assert_eq!(msgpack_value_len(&bin), Err(MsgpackError::NotJson));
            assert_eq!(
                msgpack_value_len(&[0x91, 0xd4, 1, 0]),
                Err(MsgpackError::NotJson)
            );
            assert_eq!(msgpack_value_len(&[0xc7, 0, 0]), Err(MsgpackError::NotJson));

            // {0: 1}, {nil: 1}, {"a": {1: 2}}, {[]: 1}
            assert_eq!(
                msgpack_value_len(&[0x81, 0x00, 0x01]),
                Err(MsgpackError::NonStringKey)
            );
            assert_eq!(
                msgpack_value_len(&[0x81, 0xc0, 0x01]),
                Err(MsgpackError::NonStringKey)
            );
            assert_eq!(
                msgpack_value_len(&[0x81, 0xa1, b'a', 0x81, 0x01, 0x02]),
                Err(MsgpackError::NonStringKey)
            );
            assert_eq!(
                msgpack_value_len(&[0x81, 0x90, 0x01]),
                Err(MsgpackError::NonStringKey)
            );
            // {bin "a": 1}
            assert_eq!(
                msgpack_value_len(&[0x81, 0xc4, 1, b'a', 0x01]),
                Err(MsgpackError::NonStringKey)
            );
            // Values after a nested map are values again: {"a": {}, "b": [1]}
            let ok = [0x82, 0xa1, b'a', 0x80, 0xa1, b'b', 0x91, 0x01];
            assert_eq!(msgpack_value_len(&ok), Ok(ok.len()));
        }

        #[test]
        fn malformed() {
            assert_eq!(msgpack_value_len(&[]), Err(MsgpackError::Truncated));
            assert_eq!(msgpack_value_len(&[0xc1]), Err(MsgpackError::Reserved));
            assert_eq!(
                msgpack_value_len(&[0xa5, b'a']),
                Err(MsgpackError::Truncated)
            );
            assert_eq!(
                msgpack_value_len(&[0x92, 0x01]),
                Err(MsgpackError::Truncated)
            );
            assert_eq!(
                msgpack_value_len(&[0xdf, 0xff, 0xff, 0xff, 0xff]),
                Err(MsgpackError::Truncated)
            );
            assert_eq!(
                msgpack_value_len(&[0xdb, 0xff, 0xff, 0xff, 0xff, b'a']),
                Err(MsgpackError::Truncated)
            );

            let deep = [vec![0x91; 100_000], vec![0xc0]].concat();
            assert_eq!(msgpack_value_len(&deep), Err(MsgpackError::TooDeep));
            let just_fits = [vec![0x91; 128], vec![0xc0]].concat();
            assert_eq!(msgpack_value_len(&just_fits), Ok(just_fits.len()));
        }
    }
}
