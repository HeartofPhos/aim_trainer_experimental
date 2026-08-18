use std::ops::{Deref, DerefMut};

use bevy_ecs::prelude::*;
use rand::{SeedableRng, rngs::Xoshiro128PlusPlus};

#[derive(Resource)]
pub struct Random(Xoshiro128PlusPlus);

impl Deref for Random {
    type Target = Xoshiro128PlusPlus;
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
    type Seed = <Xoshiro128PlusPlus as SeedableRng>::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        Random(Xoshiro128PlusPlus::from_seed(seed))
    }
}
