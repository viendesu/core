use eva::{
    auto_impl, rand,
    rand::Rng as _,
    time::Clock,
};

use viendesu_protocol::types::entity::{Id, IsEntityId, Kind, Metadata, SingleKindId};

use crate::rt;

#[cfg(feature = "tokio")]
pub mod real;

#[cfg(feature = "test-util")]
pub mod testing;

eva::trait_set! {
    pub trait Traits = Send + Sync;
}

pub trait Demiurge: Traits {
    type World: WorldMut + 'static;

    fn make_world(&self) -> World<Self::World>;
}

#[auto_impl(&, &mut)]
pub trait WorldRef: Traits + Clock {
    fn rt(&self) -> impl rt::RtRef;
}

#[auto_impl(&mut)]
pub trait WorldMut: WorldRef {
    fn rng(&mut self) -> impl rand::Rng;

    fn rt_mut(&mut self) -> impl rt::RtMut;
}

pub struct World<W>(pub W);

macro_rules! narrow {
    ($($method:ident : $Trait:ty),* $(,)?) => {eva::paste! {$(
        impl<W: WorldMut> World<W> {
            pub const fn [<$method _mut>](&mut self) -> &mut impl $Trait {
                &mut self.0
            }
        }

        impl<W: WorldRef> World<W> {
            pub const fn $method(&self) -> &impl $Trait {
                &self.0
            }
        }
    )*}};
}

narrow! {
   clock: Clock,
}

impl<W: WorldRef> World<W> {
    pub fn rt(&self) -> impl rt::RtRef {
        self.0.rt()
    }
}

impl<W: WorldMut> World<W> {
    pub fn rng(&mut self) -> impl rand::Rng {
        self.0.rng()
    }

    pub fn rt_mut(&mut self) -> impl rt::RtMut {
        self.0.rt_mut()
    }

    pub fn mut_(&mut self) -> World<&mut W> {
        World(&mut self.0)
    }

    pub fn generate_raw_id(&mut self, metadata: Metadata) -> Id {
        let millis = self
            .0
            .get()
            .as_millis()
            .saturating_sub(Id::TIMESTAMP_OFFSET);
        let random: u128 = self.0.rng().random();
        Id::from_parts(millis, random, metadata)
    }

    #[track_caller]
    pub fn generate_id_meta<I: IsEntityId>(&mut self, metadata: Metadata) -> I {
        let id = self.generate_raw_id(metadata);
        I::from_generic(id).expect("entity kind does not match the typed Id")
    }

    pub fn generate_id_with<I: IsEntityId>(&mut self, kind: Kind) -> I {
        self.generate_id_meta(Metadata::new(kind, 0))
    }

    pub fn generate_id<I: SingleKindId>(&mut self) -> I {
        self.generate_id_meta(Metadata::new(I::KIND, 0))
    }
}
