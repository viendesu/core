//! # Generic entity identifier
//!
//! Used to identify entities in the system. Should be treated as 128bits of randomness. Unique and stable, thus
//! it should be preferred over slugs (such as nicknames) for referring to entities.
//!
//! # Schema
//!
//! Schema is not guaranteed, can be changed any time. From highest to lowest bits.
//! ```plaintext
//!     Bits
//!  52  66  10
//! +---+---+---+
//! | T | R | M |
//! +---+---+---+
//! ```
//!
//! 1. T - timestamp in milliseconds
//! 2. R - randomness, non-zero
//! 3. M - metadata
//!
//! Thus, identifier has some kind of monotonicity and never zero. Strictly speaking, two generated IDs are
//! not guaranteed to be in order "first created > created after", it's actually "first >= after".
//!
//! The format takes inspiration from the ULID, slightly modifying it for our purposes. The exact modification is
//! metadata at the lowest bits, instead of additional 16bits of randomness, we take that bits for storing metadata, 66bits
//! is still enormous amount of possible values in a millisecond. Especially since identifiers comparison includes
//! comparing metadata, thus identifier is equal only if all of 128bits matches.
//!
//! Metadata
//! ```plaintext
//!    Bits
//!   4   6
//! +---+---+
//! | D | K |
//! +---+---+
//! ```
//! 1. D - data specific to each kind, can be randomness.
//! 2. K - Kind. Entity kind to which ID refers.
//!
//! # Encoding
//!
//! TODO.

use eva::{
    data, int,
    rand::Rng as _,
    str,
    str::{FixedUtf8, HasPattern, Seq},
    time::{Clock as _, Timestamp},
};

use std::{
    borrow::Cow,
    fmt, mem,
    num::NonZeroU128,
    slice,
    str::{FromStr, from_utf8_unchecked},
};

use crate::world::{World, WorldMut};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};

#[data(
    copy,
    display("provided number of steps would overflow capacity of the identifier")
)]
pub struct IdCapacityOverflow;

pub trait IsEntityId:
    for<'de> Deserialize<'de> + Serialize + fmt::Display + FromStr<Err = ParseError> + Copy + Eq + Ord
{
    fn from_generic(generic: Id) -> Option<Self>;
    fn to_str(&self) -> StrId;
    fn into_inner(&self) -> NonZeroU128;
}

/// Entity kind.
#[data(ord, copy, display(name))]
#[derive(Hash)]
#[repr(u8)]
pub enum Kind {
    Session = 0,
    File,
    Upload,
    AuthSession,
    User,
    Author,
    Game,
    Tag,
    Badge,
    Message,
    Thread,
    Board,
}

const _: () = {
    if Kind::MAX as u8 >= (1 << 6_u8) {
        panic!("Number of kinds exceed limits of 6bits");
    }
};

impl Kind {
    pub const MIN: Self = Self::Session;
    pub const MAX: Self = Self::Board;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Metadata(u16);

impl Metadata {
    const fn validate(v: u16) -> bool {
        let kind = v & 0b111111;
        kind <= Kind::MAX as u16
    }

    pub const fn new(kind: Kind, data: u8) -> Self {
        let kind = kind as u16;
        let data = (data & 0b1111) as u16;

        Self((data << 6) | kind)
    }

    pub const fn repr(self) -> u16 {
        self.0
    }

    pub const fn kind(self) -> Kind {
        unsafe { mem::transmute::<u8, Kind>((self.0 & 0b111111) as u8) }
    }

    pub const fn data(self) -> u8 {
        (self.0 >> 6) as u8
    }
}

#[doc(hidden)]
pub struct DisplayArray<'a, T>(pub &'a [T]);

impl<T: fmt::Display> fmt::Display for DisplayArray<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;

        for value in self.0 {
            fmt::Display::fmt(value, f)?;
        }

        f.write_str("]")
    }
}

#[data(copy, error)]
pub enum ParseError {
    #[display("unexpected entity kinds {got}, expected {}", DisplayArray(&[expected]))]
    UnexpectedKind { expected: Kind, got: Kind },
    #[display("provided identifier is malformed")]
    Malformed,
    #[display("invalid string length")]
    Length,
    #[display("invalid char")]
    Char,
}

impl From<str::ParseError> for ParseError {
    fn from(value: str::ParseError) -> Self {
        match value {
            str::ParseError::Length => Self::Length,
            str::ParseError::Char => Self::Char,
        }
    }
}

#[int(u8, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_')]
pub enum Char {}
eva::single_ascii_char!(Char);

impl Char {
    pub const NULL: Self = Self::VARIANTS[0];
}

unsafe impl FixedUtf8 for Char {}

#[str(fixed(error = eva::str::ParseError))]
struct StrIdBuf(Char, Seq<22, Char>);

impl StrIdBuf {
    const fn to_array(self) -> [Char; 23] {
        unsafe { mem::transmute::<Self, [Char; 23]>(self) }
    }

    const fn as_mut_array<'this>(&'this mut self) -> &'this mut [Char; 23] {
        unsafe { mem::transmute::<&'this mut Self, &'this mut [Char; 23]>(self) }
    }
}

/// String representation of the [`Id`].
#[str(custom, copy)]
pub struct StrId {
    buf: StrIdBuf,
    len: u8,
}

impl HasPattern for StrId {
    #[inline]
    fn pat_into(buf: &mut String) {
        Char::pat_into(buf);
        buf.push_str("{1,23}");
    }
}

impl StrId {
    pub const fn len(&self) -> u8 {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn as_str(&self) -> &str {
        unsafe {
            from_utf8_unchecked(slice::from_raw_parts(
                (&raw const self.buf).cast(),
                self.len as usize,
            ))
        }
    }
}

impl FromStr for StrId {
    type Err = ParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseError as E;

        let bytes = s.as_bytes();
        if bytes.is_empty() || bytes.len() > 23 {
            return Err(E::Length);
        }

        let mut buf = StrIdBuf(Char::NULL, Seq([Char::NULL; 22]));
        let array = buf.as_mut_array();
        let mut idx = 0_usize;
        let mut len = 0_u8;

        for &byte in bytes {
            array[idx] = Char::new(byte).ok_or(E::Char)?;
            idx += 1;
            len += 1;
        }

        Ok(Self { buf, len })
    }
}

#[doc(inline)]
pub use crate::_define_eid as define_eid;

#[doc(inline)]
pub use crate::_match_kind as match_;

#[macro_export]
#[doc(hidden)]
macro_rules! _if_tail_exists {
    ([$($Inp:ident)+] => {$($True:tt)*}; _ => {$($False:tt)*};) => {$($True)*};
    ([] => {$($True:tt)*}; _ => {$($False:tt)*};) => {$($False)*};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _use_match_arms_ty {
    (@ [$($Acc:ident)*] $Last:ident ;; as $As:ident) => {$crate::_priv::paste! { use $($Acc ::)* [<$Last _Match>] as $As; }};
    (@ [$($Acc:ident)*] $Hd:ident $($Tail:ident)+ ;; as $As:ident) => { $crate::_use_match_arms_ty!(@ [$($Acc)* $Hd] $($Tail)+ ;; as $As) };
    ($($List:ident)+ ;; as $As:ident) => { $crate::_use_match_arms_ty!(@ [] $($List)+ ;; as $As) };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _match_kind {
    ($id:expr, $($IdT:ident)::+ => {$(
        $Kind:ident => $Code:expr
    ),* $(,)?}) => {$crate::_priv::paste! {
        {
            $crate::_use_match_arms_ty!($($IdT)* ;; as __Match);

            {$(
                use $crate::types::entity::Kind::$Kind as _;
            )*}

            $id.match_(__Match {$(
                [<$Kind:snake>]: move || $Code
            ),*})
        }
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _define_eid {
    ($(
        $(#[$outer_meta:meta])*
        $vis:vis struct $Name:ident($KindHd:ident $(| $KindTs:ident)*);
    )*) => {$crate::_priv::paste! {$(
        $(#[$outer_meta])*
        #[::eva::data(copy, ord, not(Deserialize), display("{_0}"))]
        #[derive(Hash)]
        #[repr(transparent)]
        $vis struct $Name($crate::types::entity::Id);

        #[allow(non_camel_case_types, dead_code)]
        $vis struct [<$Name _Match>] <$KindHd $(, $KindTs)*> {
            $vis [<$KindHd:snake>]: $KindHd,
            $(
                $vis [<$KindTs:snake>]: $KindTs
            ),*
        }

        const _: () = {
            use $crate::{
                world::{World, WorldMut},
                types::entity,
                _if_tail_exists
            };
            use ::core::{
                result::Result,
                cmp::PartialEq,
                num::NonZeroU128,
                ops::Deref,
                str::FromStr,
            };
            use ::eva::{
                zst_error,
                _priv::serde::{
                    Deserialize,
                    Deserializer,
                    de,
                },
            };

            impl entity::IsEntityId for $Name {
                fn to_str(&self) -> entity::StrId {
                    self.0.to_str()
                }

                fn into_inner(&self) -> NonZeroU128 {
                    self.0.into_inner()
                }

                fn from_generic(v: entity::Id) -> Option<Self> {
                    Self::from_generic(v)
                }
            }

            impl Deref for $Name {
                type Target = entity::Id;

                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl FromStr for $Name {
                type Err = entity::ParseError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let id: entity::Id = s.parse()?;
                    Self::from_generic(id).ok_or(entity::ParseError::UnexpectedKind {
                        // TODO(MKS-5): enrich.
                        expected: entity::Kind::$KindHd,
                        got: id.metadata().kind(),
                    })
                }
            }

            impl PartialEq<NonZeroU128> for $Name {
                fn eq(&self, other: &NonZeroU128) -> bool {
                    self.0.into_inner() == *other
                }
            }

            impl PartialEq<u128> for $Name {
                fn eq(&self, other: &u128) -> bool {
                    self.0.into_inner().get() == *other
                }
            }

            impl PartialEq<entity::Id> for $Name {
                fn eq(&self, other: &entity::Id) -> bool {
                    self.0 == *other
                }
            }

            impl<'de> Deserialize<'de> for $Name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    let id = entity::Id::deserialize(deserializer)?;
                    Self::from_generic(id).ok_or_else(|| de::Error::custom(zst_error!(
                        "unexpected entity kind, expected {}",
                        entity::DisplayArray(&[entity::Kind::$KindHd $(, entity::Kind::$KindTs)*]),
                    )))
                }
            }

            impl $Name {
                pub const fn raw_id(self) -> entity::Id {
                    self.0
                }

                pub const fn is_valid_kind(kind: entity::Kind) -> bool {
                    matches!(kind, entity::Kind::$KindHd $(| entity::Kind::$KindTs)*)
                }

                /// Create identifier from the generic entity id.
                pub const fn from_generic(entity: entity::Id) -> Option<Self> {
                    if Self::is_valid_kind(entity.metadata().kind()) {
                        Some(Self(entity))
                    } else {
                        None
                    }
                }

                #[track_caller]
                pub const fn next_gt(self, steps: u128) -> Self {
                    match self.try_next_gt(steps) {
                        Ok(r) => r,
                        Err(..) => panic!("id capacity overflow")
                    }
                }

                pub const fn try_next_gt(self, steps: u128) -> Result<Self, entity::IdCapacityOverflow> {
                    let raw = self.raw_id();
                    match raw.try_next_gt(steps) {
                        Ok(r) => Ok(Self(r)),
                        Err(e) => Err(e),
                    }
                }

                pub fn match_<__FnOut, $KindHd $(, $KindTs)*>(self, arms: [<$Name _Match>] <$KindHd $(, $KindTs)*>) -> __FnOut
                where
                    $KindHd: FnOnce() -> __FnOut,
                    $($KindTs: FnOnce() -> __FnOut),*
                {
                    use entity::Kind as K;

                    match self.raw_id().metadata().kind() {
                        K::$KindHd => (arms.[<$KindHd:snake>])()
                        $(, K::$KindTs => (arms.[<$KindTs:snake>])())*
                        ,
                        _ => unsafe { core::hint::unreachable_unchecked() }
                    }
                }

                /// Convert identifier to the str.
                pub const fn to_str(self) -> entity::StrId {
                    self.0.to_str()
                }

                _if_tail_exists! {
                    [$($KindTs)*] => {
                        /// Generate new identifier.
                        ///
                        /// # Errors
                        ///
                        /// panics if passed wrong kind.
                        #[track_caller]
                        pub fn generate<W: WorldMut>(w: World<W>, kind: entity::Kind) -> Self {
                            if Self::is_valid_kind(kind) {
                                let id = entity::Id::generate(w, entity::Metadata::new(kind, 0));
                                Self(id)
                            } else {
                                panic!(
                                    "passed wrong kind for `{}`, available: [{}]",
                                    stringify!($Name),
                                    entity::DisplayArray(&[entity::Kind::$KindHd $(, entity::Kind::$KindTs)*])
                                )
                            }
                        }
                    };
                    _ => {
                        /// Minimal ID in sense of ordering. No ID is lesser than this one.
                        pub const MIN: Self = Self(entity::Id::from_parts(0, 0, entity::Metadata::new(entity::Kind::$KindHd, 0)));

                        /// Maximal ID in sense of ordering. No ID could be greater than this one.
                        pub const MAX: Self = Self(entity::Id::from_parts(u64::MAX, u128::MAX, entity::Metadata::new(entity::Kind::$KindHd, u8::MAX)));

                        /// Generate new identifier.
                        pub fn generate<W: WorldMut>(w: World<W>) -> Self {
                            let id = entity::Id::generate(w, entity::Metadata::new(entity::Kind::$KindHd, 0));
                            Self(id)
                        }
                    };
                }
            }
        };
    )*}};
}

/// Generic entity identifier.
#[data(copy, ord, not(serde, schemars, Debug), display("{}", self.to_str()))]
#[derive(Hash)]
pub struct Id(NonZeroU128);

impl IsEntityId for Id {
    fn from_generic(generic: Id) -> Option<Self> {
        Some(generic)
    }

    fn to_str(&self) -> StrId {
        Id::to_str(*self)
    }

    fn into_inner(&self) -> NonZeroU128 {
        self.0
    }
}

impl Id {
    pub const TIMESTAMP_OFFSET: u64 = 1735678800000;
    pub const MIN: Self = Self::from_parts(0, 0, Metadata::new(Kind::MIN, 0));
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.metadata()
                .kind()
                .as_static_str(Some(eva::generic::Case::Snake)),
            self.to_str()
        )
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.to_str().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let res = if deserializer.is_human_readable() {
            let s = <&'de str as Deserialize<'de>>::deserialize(deserializer)?;
            Self::parse(StrId::from_str(s).map_err(de::Error::custom)?)
                .map_err(de::Error::custom)?
        } else {
            let repr = u128::deserialize(deserializer)?;
            Self::from_repr(repr).ok_or_else(|| serde::de::Error::custom(ParseError::Malformed))?
        };

        Ok(res)
    }
}

impl JsonSchema for Id {
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::Id"))
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Id")
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "anyOf": [
                {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": u128::MAX,
                    "description": "integer representation of the identifier, only in binary formats"
                },
                {
                    "type": "string",
                    "pattern": StrId::regex_pat_fullmatch(),
                    "minLength": 1,
                    "maxLength": 23,
                    "description": "unique entity identifier, only in human-readable formats"
                }
            ]
        })
    }
}

impl FromStr for Id {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: StrId = s.parse()?;
        Ok(Self::parse(s)?)
    }
}

impl Id {
    pub const fn to_str(self) -> StrId {
        let mut buf = StrIdBuf(Char::NULL, Seq([Char::NULL; 22]));
        let array = buf.as_mut_array();
        let mut len = 0_u8;

        let mut value = self.0.get();
        let base = Char::VARIANTS.len() as u128;
        let idx = value.ilog(base) as usize;

        loop {
            array[idx - len as usize] = Char::VARIANTS[(value % base) as usize];
            value /= base;
            len += 1;
            if value == 0 {
                break;
            }
        }

        StrId { buf, len }
    }

    pub const fn from_repr(x: u128) -> Option<Self> {
        let Some(x) = NonZeroU128::new(x) else {
            return None;
        };
        if !Metadata::validate((x.get() & 0x3FF) as u16) {
            return None;
        }

        Some(Self(x))
    }

    pub fn parse(s: StrId) -> Result<Self, ParseError> {
        let mut res = 0_u128;
        let mut idx = 0_usize;
        let array = s.buf.to_array();

        while idx != s.len() as usize {
            res = res
                .checked_mul(Char::VARIANTS.len() as u128)
                .ok_or(ParseError::Malformed)?;
            res = res
                .checked_add(array[idx].nth() as u128)
                .ok_or(ParseError::Malformed)?;
            idx += 1;
        }

        Self::from_repr(res).ok_or(ParseError::Malformed)
    }

    pub const fn timestamp(self) -> Timestamp {
        Timestamp::from_millis(self.timestamp_ms())
    }

    pub const fn timestamp_ms_rel(self) -> u64 {
        self.timestamp_ms() - Self::TIMESTAMP_OFFSET
    }

    pub const fn timestamp_ms(self) -> u64 {
        (self.0.get() >> 76) as u64 + Self::TIMESTAMP_OFFSET
    }

    pub const fn metadata(self) -> Metadata {
        Metadata((self.0.get() & 0x3FF) as u16)
    }

    pub const fn with_metadata(self, new: Metadata) -> Self {
        Self(unsafe { NonZeroU128::new_unchecked((self.0.get() & !0x3FF) | new.repr() as u128) })
    }

    /// Same as [`Id::try_next_gt`], but panics on overflow.
    #[track_caller]
    pub const fn next_gt(self, steps: u128) -> Self {
        if let Ok(this) = self.try_next_gt(steps) {
            this
        } else {
            panic!("steps would overflow")
        }
    }

    /// Get entity id which'd be greater than `steps` of next identifiers.
    pub const fn try_next_gt(self, mut steps: u128) -> Result<Self, IdCapacityOverflow> {
        const RAND_QUOTA: u128 = (1 << 66) - 1;

        let mut ts = self.timestamp_ms_rel();
        let rand = self.random().get();
        let meta = self.metadata();
        let rand_quota_left = RAND_QUOTA - rand;

        if rand_quota_left >= steps {
            return Ok(Self::from_parts(ts, rand + steps, meta));
        }

        // rand_quota_left < steps
        ts += 1;
        steps -= rand_quota_left;

        if RAND_QUOTA >= steps {
            return Ok(Self::from_parts(ts, steps, meta));
        }

        // RAND_QUOTA < steps
        // So, at minimum we're `2^72 + 1` steps ahead, one more
        // jump and we got the maximum value.

        steps -= RAND_QUOTA;
        ts += 1;

        if RAND_QUOTA >= steps {
            Ok(Self::from_parts(ts, steps, meta))
        } else {
            // We're already `2^144 + 1` steps ahead, there's no more
            // values.
            Err(IdCapacityOverflow)
        }
    }

    /// Get random bits of the identifier.
    pub const fn random(self) -> NonZeroU128 {
        unsafe { NonZeroU128::new_unchecked((self.0.get() & (((1 << 66) - 1) << 10)) >> 10) }
    }

    /// Construct ID from parts.
    pub const fn from_parts(mut millis: u64, mut random: u128, metadata: Metadata) -> Self {
        millis &= (1 << 52) - 1;
        random &= (1 << 66) - 1;
        if random == 0 {
            random = 1;
        }

        let mut result = metadata.repr() as u128;
        result |= random << 10;
        result |= (millis as u128) << 76;

        Self(unsafe { NonZeroU128::new_unchecked(result) })
    }

    /// Generate new identifier.
    pub fn generate<W: WorldMut>(mut world: World<W>, metadata: Metadata) -> Self {
        let millis = world
            .clock()
            .get()
            .as_millis()
            .saturating_sub(Self::TIMESTAMP_OFFSET);
        let random: u128 = world.rng().random();

        Self::from_parts(millis, random, metadata)
    }

    /// Convert ID into its raw representation.
    pub const fn into_inner(self) -> NonZeroU128 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KIND: Kind = Kind::MIN;
    const META: Metadata = Metadata::new(KIND, 0);

    #[test]
    fn steps_simple_ordering() {
        let fst = Id::from_parts(0, 1, META);
        assert_eq!(fst.random().get(), 1);

        // 0 steps = same id.
        assert_eq!(fst.next_gt(0), fst);

        // Only random should increment.
        {
            let next = fst.next_gt(1);
            assert_eq!(next.timestamp_ms_rel(), 0);
            assert_eq!(next.random().get(), 2);
            assert_eq!(next.metadata(), META);
        }
    }

    #[test]
    fn steps_borderline_ordering() {
        const QUOTA: u128 = (1 << 66) - 1;

        let fst = Id::from_parts(0, QUOTA - 1, META);

        // random() = QUOTA is ok, should be preserved.
        {
            let next = fst.next_gt(1);
            assert_eq!(next.timestamp_ms_rel(), 0);
            assert_eq!(next.random().get(), QUOTA);
            assert_eq!(next.metadata(), META);
        }

        // Timestamp increment.
        {
            let next = fst.next_gt(2);
            assert_eq!(next.timestamp_ms_rel(), 1);
            assert_eq!(next.random().get(), 1);
            assert_eq!(next.metadata(), META);
        }

        // Random should be 2, when getting next greater id, 1 = 0
        // shouldn't be visible.
        {
            let next = fst.next_gt(3);
            assert_eq!(next.timestamp_ms_rel(), 1);
            // May be faulty due to `1 = 0` in random, MUST be worked-around
            // by implementation by removing `0` from the set of possible values
            // here.
            assert_eq!(next.random().get(), 2);
            assert_eq!(next.metadata(), META);
        }

        // Make third jump, should be correct as well.
        {
            let next = fst.next_gt(2 + QUOTA);
            assert_eq!(next.timestamp_ms_rel(), 2);
            assert_eq!(next.random().get(), 1);
            assert_eq!(next.metadata(), META);
        }

        // Same for third jump, 1 = 0 check.
        {
            let next = fst.next_gt(2 + QUOTA + 1);
            assert_eq!(next.timestamp_ms_rel(), 2);
            assert_eq!(next.random().get(), 2);
            assert_eq!(next.metadata(), META);
        }
    }

    #[test]
    fn simple_ordering() {
        assert!(Id::from_parts(0, 0, META) < Id::from_parts(1, 0, META));
    }

    #[test]
    fn converting_back() {
        let id = Id::from_parts(12345, 67890, META);
        let s = id.to_str().to_string();
        let back: Id = s.parse().unwrap();

        assert_eq!(id, back);
    }

    #[test]
    fn from_parts_basic() {
        let ts = 123456789_u64;
        let rand = 987654321_u128;
        let meta = Metadata::new(Kind::User, 5);

        let id = Id::from_parts(ts, rand, meta);

        assert_eq!(id.timestamp_ms_rel(), ts);
        assert_eq!(id.random().get(), rand);
        assert_eq!(id.metadata(), meta);
        assert_eq!(id.metadata().kind(), Kind::User);
        assert_eq!(id.metadata().data(), 5);
    }

    #[test]
    fn from_parts_max_timestamp() {
        // 52 bits max
        let max_ts = (1_u64 << 52) - 1;
        let id = Id::from_parts(max_ts, 1, META);

        assert_eq!(id.timestamp_ms_rel(), max_ts);
    }

    #[test]
    fn from_parts_timestamp_overflow() {
        // Values beyond 52 bits should be masked
        let overflow_ts = (1_u64 << 52) + 100;
        let id = Id::from_parts(overflow_ts, 1, META);

        // Should be masked to 52 bits
        assert_eq!(id.timestamp_ms_rel(), 100);
    }

    #[test]
    fn from_parts_max_random() {
        // 66 bits max
        let max_rand = (1_u128 << 66) - 1;
        let id = Id::from_parts(0, max_rand, META);

        assert_eq!(id.random().get(), max_rand);
    }

    #[test]
    fn from_parts_random_overflow() {
        // Values beyond 66 bits should be masked
        let overflow_rand = (1_u128 << 66) + 42;
        let id = Id::from_parts(0, overflow_rand, META);

        // Should be masked to 66 bits
        assert_eq!(id.random().get(), 42);
    }

    #[test]
    fn from_parts_zero_random_becomes_one() {
        // Zero random should become 1 (to ensure NonZero)
        let id = Id::from_parts(0, 0, META);

        assert_eq!(id.random().get(), 1);
    }

    #[test]
    fn metadata_all_kinds() {
        // TODO: write test that will check kind is correctly returned.
    }

    #[test]
    fn metadata_data_values() {
        // 4 bits = 0-15
        for data in 0..16_u8 {
            let meta = Metadata::new(Kind::User, data);
            let id = Id::from_parts(0, 1, meta);

            assert_eq!(id.metadata().data(), data);
        }
    }

    #[test]
    fn metadata_data_overflow() {
        // Values beyond 4 bits should be masked
        let meta = Metadata::new(Kind::User, 0b11110101);
        // Only lowest 4 bits (0101 = 5) should be kept
        assert_eq!(meta.data(), 5);
    }

    #[test]
    fn from_parts_preserves_all_components() {
        // Test with all components at interesting values
        let ts = 0xABCDE_u64;
        let rand = 0x123456789ABCDEF_u128;
        let meta = Metadata::new(Kind::Board, 0xF);

        let id = Id::from_parts(ts, rand, meta);

        assert_eq!(id.timestamp_ms_rel(), ts);
        assert_eq!(id.random().get(), rand);
        assert_eq!(id.metadata().kind(), Kind::Board);
        assert_eq!(id.metadata().data(), 0xF);
    }
}
