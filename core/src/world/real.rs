use crate::{
    rt,
    world::{Demiurge, World, WorldMut, WorldRef},
};

use eva::{
    rand,
    time::{Clock, RealTime, Timestamp},
};

pub struct God;

impl Demiurge for God {
    type World = RealWorld;

    fn make_world(&self) -> World<Self::World> {
        World(RealWorld)
    }
}

pub struct RealWorld;

impl WorldRef for RealWorld {
    fn rt(&self) -> impl rt::RtRef {
        rt::TokioRt
    }
}

impl WorldMut for RealWorld {
    fn rng(&mut self) -> impl rand::Rng {
        rand::rng()
    }

    fn rt_mut(&mut self) -> impl rt::RtMut {
        rt::TokioRt
    }
}

impl Clock for RealWorld {
    fn get(&self) -> Timestamp {
        Clock::get(&RealTime::default())
    }
}
