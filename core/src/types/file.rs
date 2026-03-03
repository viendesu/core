//! # File identifier
//!
//! Basically same as [`entity::Id`], but additionally stores server where file is located in
//! two lowest bits of random.

use std::{
    mem,
    num::{NonZeroU64, NonZeroU128},
    str::FromStr,
};

use crate::{
    types::entity,
    world::{World, WorldMut},
};
use eva::{data, hash::blake3, int, rand::Rng, str, str::ToCompactString, time::Clock};

pub type Hash = blake3::Hash;

pub mod game_file;
pub mod image;

#[data]
pub struct FileInfo {
    pub id: Id,
    pub class: Class,
    pub size: NonZeroU64,
}

#[data(copy, ord, display(name))]
pub enum ClassKind {
    Image,
    GameFile,
}

#[data]
pub enum Class {
    Image(image::ImageInfo),
    GameFile(game_file::GameFileInfo),
}

const BASE_NAME_PAT: &str = r"[^/\]+";

/// Base file name.
#[str(custom)]
#[derive(schemars::JsonSchema)]
pub struct BaseName(
    #[schemars(regex(pattern = BASE_NAME_PAT), length(max = BaseName::MAX_LEN))] str::CompactString,
);

impl str::HasPattern for BaseName {
    fn pat_into(buf: &mut String) {
        buf.push_str(BASE_NAME_PAT);
    }
}

impl BaseName {
    pub const MAX_LEN: usize = 4096;

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for BaseName {
    type Err = str::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Considerably long file name.
        if s.len() >= Self::MAX_LEN {
            return Err(str::ParseError::Length);
        }
        if s.chars().any(|c| matches!(c, '/' | '\\')) {
            return Err(str::ParseError::Char);
        }

        Ok(Self(s.into()))
    }
}

/// ID of the server.
#[int(u8, 0..8)]
#[derive(Hash)]
pub enum Server {}

impl Server {
    pub const fn try_from_ascii_repr(val: u8) -> Option<Self> {
        if val < b'a' {
            return None;
        }

        Self::new(val - b'a')
    }

    pub const fn ascii_repr(self) -> u8 {
        b'a' + (self as u8)
    }
}

entity::define_eid! {
    pub struct Id(File);
}

impl Id {
    pub fn new<W: WorldMut>(mut w: World<W>, server: Server) -> Self {
        Self::from_parts(w.clock().get().as_millis(), w.rng().random(), server)
    }

    pub const fn from_parts(millis: u64, random: u128, server: Server) -> Self {
        let id = entity::Id::from_parts(
            millis,
            random,
            entity::Metadata::new(entity::Kind::File, server as u8),
        );
        Self(id)
    }

    pub const fn server(self) -> Server {
        unsafe { mem::transmute::<u8, Server>(self.0.metadata().data() & 0b111) }
    }

    pub const fn into_inner(self) -> NonZeroU128 {
        self.0.into_inner()
    }
}
