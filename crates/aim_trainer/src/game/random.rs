use bevy_ecs::prelude::*;
use rand::{SeedableRng, rngs::Xoshiro256PlusPlus};
use std::ops::{Deref, DerefMut};

type RandImpl = Xoshiro256PlusPlus;

#[derive(Resource)]
pub struct Random(RandImpl);

impl Deref for Random {
    type Target = RandImpl;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Random {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SeedableRng for Random {
    type Seed = <RandImpl as SeedableRng>::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        Random(RandImpl::from_seed(seed))
    }
}
