use crate::logic::challenge::ChallengeSet;
use avian3d::schedule::PhysicsSystems;
use bevy::prelude::*;

pub mod challenge;
pub mod character;
pub mod health;
pub mod level;
pub mod shape;
pub mod spawn;
pub mod targeter;
pub mod team;
pub mod weapon;

pub fn plugin(app: &mut App) {
    app.add_plugins(challenge::plugin);
    app.add_plugins(character::plugin);
    app.add_plugins(health::plugin);
    app.add_plugins(level::plugin);
    app.add_plugins(shape::plugin);
    app.add_plugins(spawn::plugin);
    app.add_plugins(weapon::plugin);

    app.configure_sets(
        FixedUpdate,
        (
            ChallengeSet::Accumulate,
            ChallengeSet::Update,
            SpawnSet::Spawn,
            SpawnSet::Move,
            WeaponSet,
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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WeaponSet;

#[derive(Component, Clone, Copy)]
pub struct TimeFactor(pub f32);
