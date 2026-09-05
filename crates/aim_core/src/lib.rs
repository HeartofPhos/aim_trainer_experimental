use crate::{
    layers::CollisionLayersExt,
    logic::{
        TimeFactor,
        challenge::{
            Challenge, ChallengeValue, CollectableSensor, ScalableComponent, challenge_bundle,
        },
        character::{
            auto_driver::AutoDriver,
            character_controller::{CharacterController, GroundDetection},
            input_driver::InputDriver,
            movement::{FacingOf, MovementProfile},
        },
        health::health_bundle,
        level::{BrushDef, PrimitiveCache},
        shape::Shape,
        spawn::{SpawnLookup, Spawned, Spawner},
        targeter::TargeterOf,
        team::Team,
        weapon::weapon_bundle,
    },
};
use avian3d::prelude::*;
use bevy::{ecs::system::RunSystemOnce, log::LogPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_rand::plugin::EntropyPlugin;
use schema::{Scenario, SpawnGroup, SpawnRules};
use std::time::Duration;

mod layers;
pub mod logic;
mod utils;

#[derive(Component)]
pub struct Camera;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Bot;

#[derive(Component)]
struct Collectable;

#[derive(Component, Clone, Copy)]
pub enum Render {
    Shape,
    Radius,
}

pub type AimRng = bevy_rand::prelude::WyRand;

pub struct AimCore {
    pub scenario: Scenario,
    pub challenge: ChallengeValue,
    pub timestep: Duration,
    pub seed: <AimRng as rand::SeedableRng>::Seed,
}

impl AimCore {
    pub fn build(self) -> App {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(LogPlugin::default());
        app.add_plugins(TransformPlugin);
        app.insert_resource(Time::<Fixed>::from_duration(self.timestep));
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(EntropyPlugin::<AimRng>::with_seed(self.seed));
        app.add_plugins(logic::plugin);

        app.init_resource::<PrimitiveCache>();
        app.init_resource::<SpawnLookup>();
        app.insert_resource(Challenge::new(self.challenge));

        let mut time = app.world_mut().resource_mut::<TimeUpdateStrategy>();
        *time = TimeUpdateStrategy::ManualDuration(self.timestep);

        app.world_mut()
            .run_system_once_with(load_scenario, self.scenario)
            .expect("failed to load scenario");

        app.finish();
        app.cleanup();

        app
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Input {
    pub look: Vec2,
    pub movement: Vec3,
    pub fire: bool,
}

fn load_scenario(
    In(scenario): In<Scenario>,
    mut commands: Commands,
    mut spawn_lookup: ResMut<SpawnLookup>,
) {
    for brush in scenario.level.brush_list {
        match brush.def {
            schema::BrushDef::Spawn { group } => {
                spawn_lookup.push(group, brush.transform);
            }
            schema::BrushDef::Primitive { .. } => (),
        }

        commands.spawn((
            Transform {
                translation: brush.transform.translation,
                rotation: brush.transform.rotation,
                scale: brush.transform.scale,
            },
            RigidBody::Static,
            CollisionLayers::brush(&brush.def),
            BrushDef(brush.def),
        ));
    }

    fn character(
        mut commands: Commands,
        entity: Entity,
        character: schema::CharacterTemplate,
        movement: schema::MovementProfile,
        size_scaling: Option<schema::ChallengeScaling>,
        time_scaling: Option<schema::ChallengeScaling>,
    ) -> Entity {
        let shape = Shape::from(character.shape);

        let entity = commands
            .entity(entity)
            .insert((
                shape.with_scaling(size_scaling),
                TimeFactor(1.0).with_scaling(time_scaling),
                MovementProfile(movement),
                CharacterController,
                GroundDetection::default(),
            ))
            .id();

        let (anchor_transform, view_transform) = if let Some(eyes) = character.eyes {
            (
                Transform::from_xyz(0.0, -shape.extents().y + eyes.height, 0.0),
                Transform::from_xyz(0.0, 0.0, eyes.offset),
            )
        } else {
            (Transform::default(), Transform::default())
        };

        let anchor = commands.spawn((anchor_transform, ChildOf(entity))).id();
        let view = commands.spawn((view_transform, ChildOf(anchor))).id();

        commands.entity(anchor).insert(FacingOf(entity));

        view
    }

    commands
        .spawn(Spawner {
            spawn_rules: SpawnRules { limit: 1 },
            spawn_group: SpawnGroup(0),
        })
        .observe(move |spawned: On<Spawned>, mut commands: Commands| {
            let player = spawned.spawned;

            let view = character(
                commands.reborrow(),
                player,
                scenario.player.character,
                scenario.player.movement,
                None,
                None,
            );

            commands.entity(player).insert((
                Player,
                Team(schema::Team::PLAYER),
                Render::Shape,
                InputDriver,
                CollisionLayers::character(schema::Team::PLAYER),
                weapon_bundle(scenario.weapon),
                challenge_bundle(scenario.challenge),
            ));

            commands.entity(view).insert((Camera, TargeterOf(player)));
        });

    if let Some(bot_template) = scenario.bot_template {
        commands
            .spawn(Spawner {
                spawn_rules: bot_template.spawn_rules,
                spawn_group: SpawnGroup(1),
            })
            .observe(move |spawned: On<Spawned>, mut commands: Commands| {
                let bot = spawned.spawned;

                character(
                    commands.reborrow(),
                    bot,
                    bot_template.character,
                    bot_template.movement,
                    Some(bot_template.size_scaling),
                    Some(bot_template.time_scaling),
                );

                commands.entity(bot).insert((
                    Bot,
                    Team(schema::Team::BOT),
                    Render::Shape,
                    AutoDriver::from(bot_template.driver.clone()),
                    CollisionLayers::character(schema::Team::BOT),
                    health_bundle(bot_template.health.into()),
                ));
            });
    }

    if let Some(collectable_template) = scenario.collectable_template {
        commands
            .spawn(Spawner {
                spawn_rules: collectable_template.spawn_rules,
                spawn_group: SpawnGroup(0),
            })
            .observe(move |spawned: On<Spawned>, mut commands: Commands| {
                character(
                    commands.reborrow(),
                    spawned.spawned,
                    collectable_template.character,
                    collectable_template.movement,
                    None,
                    None,
                );

                let collectable = commands
                    .entity(spawned.spawned)
                    .insert((
                        Collectable,
                        AutoDriver::from(collectable_template.driver.clone()),
                        CollisionLayers::collectable(),
                    ))
                    .id();

                let collectable_sensor_shape = Shape::from(collectable_template.shape);
                commands.spawn((
                    CollectableSensor,
                    collectable_sensor_shape,
                    Render::Radius,
                    CollisionLayers::collectable_sensor(),
                    CollisionEventsEnabled,
                    Transform::IDENTITY,
                    ChildOf(collectable),
                ));
            });
    }
}
