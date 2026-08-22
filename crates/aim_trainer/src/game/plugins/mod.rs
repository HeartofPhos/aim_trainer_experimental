use bevy::prelude::*;
use bevy_rand::prelude::*;

pub mod character_controller;
pub mod input_driver;
pub mod movement;
pub mod spawn;

pub fn plugin(app: &mut App) {
    app.add_plugins(character_controller::plugin);
    app.add_plugins(input_driver::plugin);
    app.add_plugins(movement::plugin);
    app.add_plugins(spawn::plugin);

    app.configure_sets(
        Update,
        (
            SpawnSet::Spawn,
            SpawnSet::Move,
            InputSet,
            MovementSet,
            CharacterControllerSet,
            PhysicsSet::SyncBackend,
            PhysicsSet::StepSimulation,
            PhysicsSet::Writeback,
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSet {
    SyncBackend,
    StepSimulation,
    Writeback,
}
