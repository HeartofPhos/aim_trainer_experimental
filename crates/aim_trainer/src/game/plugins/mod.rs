use bevy_ecs::prelude::*;

use crate::game::Update;

pub mod character_controller;
pub mod input_driver;
pub mod movement;
pub mod spawn;

pub fn plugin(world: &mut World) {
    character_controller::plugin(world);
    input_driver::plugin(world);
    movement::plugin(world);
    spawn::plugin(world);

    world.get_resource_or_init::<Schedules>().configure_sets(
        Update,
        (
            SpawnSet::Spawn,
            SpawnSet::Move,
            InputSet,
            MovementSet,
            CharacterControllerSet,
        )
            .chain(),
    );
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharacterControllerSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpawnSet {
    Spawn,
    Move,
}
