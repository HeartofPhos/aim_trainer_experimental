use bevy_ecs::prelude::*;

pub fn plugin(world: &mut World) {}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
