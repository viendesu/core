use std::sync::{Arc, Mutex};

use eva::{
    fut::Fut,
    rand::{self, xoshiro},
    time,
};

use crate::rt;

pub struct World {
    rng: xoshiro::Xoshiro256StarStar,
    clock: time::Mock,
}

impl time::Clock for World {
    fn get(&self) -> time::Timestamp {
        self.clock.get()
    }
}

impl super::WorldRef for World {
    fn rt(&self) -> impl rt::RtRef {
        self
    }
}

impl super::WorldMut for World {
    fn rt_mut(&mut self) -> impl rt::RtMut {
        self
    }

    fn rng(&mut self) -> impl rand::Rng {
        &mut self.rng
    }
}

impl rt::RtRef for World {}
impl rt::RtMut for World {
    fn spawn<F: Fut<Output: Send> + 'static>(&mut self, task: F) {
        tokio::spawn(task);
    }

    async fn spawn_blocking<F: rt::BlockingFn>(&mut self, task: F) -> F::Ret {
        tokio::task::spawn_blocking(task)
            .await
            .expect("underlying task panicked")
    }
}

pub struct Demiurge {
    rng: Arc<Mutex<xoshiro::Xoshiro256StarStar>>,
    pub clock: time::Mock,
}

impl Demiurge {
    pub fn from_os_rng() -> Self {
        Self {
            rng: Arc::new(Mutex::new(rand::SeedableRng::from_os_rng())),
            clock: time::Mock::default(),
        }
    }

    pub fn new(seed: u64) -> Self {
        Self {
            rng: Arc::new(Mutex::new(rand::SeedableRng::seed_from_u64(seed))),
            clock: time::Mock::default(),
        }
    }
}

impl super::Demiurge for Demiurge {
    type World = World;

    fn make_world(&self) -> super::World<Self::World> {
        super::World(World {
            rng: {
                let mut global = self.rng.lock().unwrap();
                let rng = (*global).clone();
                // Jump to avoid overlapping.
                global.jump();
                rng
            },
            clock: self.clock.clone(),
        })
    }
}
