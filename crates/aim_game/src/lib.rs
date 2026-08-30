use crate::{
    layers::CollisionLayersExt,
    logic::{
        TimeFactor,
        auto_driver::AutoDriver,
        character_controller::{CharacterController, GroundDetection},
        health::{Health, health_bundle},
        input_driver::InputDriver,
        level::{BrushDef, PrimitiveCache},
        movement::{FacingOf, MovementProfile},
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

pub struct Game {
    app: App,
}

#[derive(Component)]
struct Camera;

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

pub type GameRng = bevy_rand::prelude::WyRand;

impl Game {
    pub fn new(scenario: Scenario, time_step: Duration) -> Self {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(LogPlugin::default());
        app.add_plugins(TransformPlugin);
        app.insert_resource(Time::<Fixed>::from_duration(time_step));
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(EntropyPlugin::<GameRng>::with_seed([0; 8]));
        app.add_plugins(logic::plugin);

        app.init_resource::<Time>();
        app.init_resource::<PrimitiveCache>();
        app.init_resource::<SpawnLookup>();

        app.world_mut()
            .run_system_once_with(load_scenario, scenario)
            .expect("failed to load scenario");

        app.finish();

        Self { app }
    }

    pub fn primitive_cache(&self) -> &PrimitiveCache {
        self.app.world().resource::<PrimitiveCache>()
    }

    pub fn level_brushes(&self, mut f: impl FnMut(&schema::BrushDef, &schema::BrushTransform)) {
        let mut query = self
            .app
            .world()
            .try_query::<(&BrushDef, &Transform)>()
            .unwrap();

        for (brush_def, transform) in query.iter(self.app.world()) {
            f(
                brush_def,
                &schema::BrushTransform {
                    translation: transform.translation,
                    rotation: transform.rotation,
                    scale: transform.scale,
                },
            );
        }
    }

    pub fn update(&mut self, delta_time: Duration, input: Input) {
        let mut time = self.app.world_mut().resource_mut::<TimeUpdateStrategy>();
        *time = TimeUpdateStrategy::ManualDuration(delta_time);

        self.app.insert_resource(input);

        self.app.update();
    }

    pub fn camera(&self) -> Result<Transform> {
        let transform = self
            .app
            .world()
            .try_query_filtered::<&GlobalTransform, With<Camera>>()
            .ok_or("failed query")?
            .single(self.app.world())?;

        Ok((*transform).into())
    }

    pub fn shapes(&self, mut f: impl FnMut(Transform, Shape, Render, Option<Health>)) {
        let mut query = self
            .app
            .world()
            .try_query_filtered::<(&GlobalTransform, &Shape, &Render, Option<&Health>), ()>()
            .unwrap();

        for (transform, shape, render, health) in query.iter(self.app.world()) {
            f((*transform).into(), *shape, *render, health.copied());
        }
    }
}

#[derive(Resource, Default, Clone, Copy)]
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
    ) -> Entity {
        let shape = Shape::from(character.shape);

        let entity = commands
            .entity(entity)
            .insert((
                shape,
                TimeFactor::default(),
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
            );

            commands.entity(player).insert((
                Player,
                Team(schema::Team::PLAYER),
                Render::Shape,
                InputDriver,
                CollisionLayers::character(schema::Team::PLAYER),
                weapon_bundle(scenario.weapon),
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
                    collectable_sensor_shape,
                    Render::Radius,
                    Sensor,
                    CollisionLayers::collectable_sensor(),
                    CollisionEventsEnabled,
                    Transform::IDENTITY,
                    ChildOf(collectable),
                ));
            });
    }
}
