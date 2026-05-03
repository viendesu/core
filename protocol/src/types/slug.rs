use std::{borrow::Cow, num::NonZeroU8, slice, str::FromStr};

use eva::{data, int, str, str::HasPattern};
use schemars::JsonSchema;

pub type LowerSlug<const MAX: usize> = Slug<MAX, LowerSlugStart, LowerSlugRest>;

pub unsafe trait SlugPart: Default + Copy + Eq + Ord {
    const TYPE_NAME: &str;

    fn from_u8(u: u8) -> Option<Self>;
    fn push_pat(into: &mut String);
}

#[data(copy, error, display(doc))]
pub enum ParseError {
    /// Invalid char at start.
    CharAtStart,
    /// Invalid char at rest.
    CharAtRest { pos: u8 },
    /// Invalid length.
    Length,
}

macro_rules! slug_part {
    ($ty:ident: $default:literal) => {
        impl Default for $ty {
            fn default() -> Self {
                Self::new($default).expect("shit happens")
            }
        }

        unsafe impl SlugPart for $ty {
            const TYPE_NAME: &str = stringify!($ty);

            fn push_pat(into: &mut String) {
                eva::push_ascii_pat!($ty, into);
            }

            fn from_u8(u: u8) -> Option<Self> {
                Self::new(u)
            }
        }
    };
}

#[int(u8, b'a'..=b'z')]
pub enum LowerSlugStart {}

slug_part!(LowerSlugStart: b'z');

#[int(u8, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-')]
pub enum LowerSlugRest {}

slug_part!(LowerSlugRest: b'z');

#[int(u8, b'A'..=b'Z' | b'a'..=b'z')]
pub enum SlugStart {}

slug_part!(SlugStart: b'Z');

#[int(u8, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_')]
pub enum SlugRest {}

slug_part!(SlugRest: b'Z');

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C, align(1))]
struct SlugBuf<const MAX: usize, Start, Rest>(Start, [Rest; MAX]);

#[str(custom, copy)]
pub struct Slug<const MAX: usize, Start: SlugPart = SlugStart, Rest: SlugPart = SlugRest> {
    buf: SlugBuf<MAX, Start, Rest>,
    len: NonZeroU8,
}

impl<const MAX: usize> Slug<MAX> {
    pub const MAX: usize = MAX;

    pub const fn lower(mut self) -> LowerSlug<MAX> {
        unsafe {
            self.buf.0 = SlugStart::new_unchecked((self.buf.0 as u8).to_ascii_lowercase());
            let mut idx = 0;

            while idx != MAX {
                let rest = self.buf.1[idx] as u8;
                self.buf.1[idx] = SlugRest::new_unchecked(rest.to_ascii_lowercase());
                idx += 1;
            }
        }
        unsafe { std::mem::transmute_copy::<Slug<MAX>, LowerSlug<MAX>>(&self) }
    }
}

impl<const MAX: usize, Start: SlugPart, Rest: SlugPart> FromStr for Slug<MAX, Start, Rest> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseError as E;

        #[expect(non_snake_case)]
        let Z = Rest::default();

        let Some((head, tail_str)) = s.split_at_checked(1) else {
            return Err(E::Length);
        };

        let head = Start::from_u8(head.as_bytes()[0]).ok_or(E::CharAtStart)?;
        let mut tail = [Z; MAX];
        let mut len = 1_u8;
        for (idx, b) in tail_str.as_bytes().iter().copied().enumerate() {
            tail[idx] = Rest::from_u8(b).ok_or(E::CharAtRest {
                pos: (idx + 1) as u8,
            })?;
            len = len.checked_add(1).ok_or(E::Length)?;
        }

        Ok(Self {
            buf: SlugBuf(head, tail),
            len: unsafe { NonZeroU8::new_unchecked(len) },
        })
    }
}

impl<const MAX: usize, Start: SlugPart, Rest: SlugPart> JsonSchema for Slug<MAX, Start, Rest> {
    fn schema_id() -> Cow<'static, str> {
        format!(
            "{}::Slug<{}, {}, {}>",
            module_path!(),
            MAX + 1,
            Start::TYPE_NAME,
            Rest::TYPE_NAME
        )
        .into()
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!(
            "Slug_{}_{}_{}",
            MAX + 1,
            Start::TYPE_NAME,
            Rest::TYPE_NAME
        ))
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let mut pat = String::from("^");
        Self::pat_into(&mut pat);
        pat.push('$');

        schemars::json_schema!({
            "type": "string",
            "description": "human and machine readable identifier",
            "minLength": 1,
            "maxLength": MAX + 1,
            "pattern": pat,
        })
    }
}

impl<const MAX: usize, Start: SlugPart, Rest: SlugPart> Slug<MAX, Start, Rest> {
    pub const fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(slice::from_raw_parts(
                (&raw const self.buf).cast(),
                self.len.get() as usize,
            ))
        }
    }
}

impl<const MAX: usize, Start: SlugPart, Rest: SlugPart> HasPattern for Slug<MAX, Start, Rest> {
    fn pat_into(buf: &mut String) {
        Start::push_pat(buf);
        Rest::push_pat(buf);

        buf.push_str("{0,");
        buf.push_str(&MAX.to_string());
        buf.push_str("}");
    }
}
