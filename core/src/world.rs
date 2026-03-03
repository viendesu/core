use eva::{auto_impl, rand, time::Clock};

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
}
