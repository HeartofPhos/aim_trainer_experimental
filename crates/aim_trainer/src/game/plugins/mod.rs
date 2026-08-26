use avian3d::schedule::PhysicsSystems;
use bevy::prelude::*;

pub mod auto_driver;
pub mod character_controller;
pub mod input_driver;
pub mod level;
pub mod movement;
pub mod shape;
pub mod spawn;

pub fn plugin(app: &mut App) {
    app.add_plugins(auto_driver::plugin);
    app.add_plugins(input_driver::plugin);
    app.add_plugins(character_controller::plugin);
    app.add_plugins(movement::plugin);
    app.add_plugins(level::plugin);
    app.add_plugins(shape::plugin);
    app.add_plugins(spawn::plugin);

    app.configure_sets(
        FixedUpdate,
        (
            SpawnSet::Spawn,
            SpawnSet::Move,
            DriverSet,
            MovementSet,
            CharacterControllerSet,
            PhysicsSystems::First,
        )
            .chain(),
    );
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharacterControllerSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DriverSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpawnSet {
    Spawn,
    Move,
}

#[derive(Component)]
pub struct TimeFactor(f32);

impl Default for TimeFactor {
    fn default() -> Self {
        Self(1.0)
    }
}
