use bevy::prelude::*;

pub fn plugin(app: &mut App) {}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
