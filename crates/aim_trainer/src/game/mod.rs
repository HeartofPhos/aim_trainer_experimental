use crate::game::{
    config::SensitivityConfig,
    layers::CollisionLayersExt,
    plugins::{
        TimeFactor,
        auto_driver::AutoDriver,
        character_controller::{CharacterController, GroundDetection},
        input_driver::InputDriver,
        level::{BrushDef, PrimitiveCache},
        movement::{FacingOf, MovementProfile},
        shape::Shape,
        spawn::{SpawnLookup, Spawned, Spawner},
    },
};
use avian3d::prelude::*;
use bevy::{log::LogPlugin, prelude::*, time::TimeUpdateStrategy};
use bevy_rand::plugin::EntropyPlugin;
use schema::{SpawnGroup, SpawnRules, Team};
use std::{path::PathBuf, time::Duration};

mod config;
mod layers;
pub mod plugins;
mod utils;

pub struct Game {
    app: App,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Camera;

#[derive(Component)]
struct Bot;

#[derive(Event)]
struct LoadScenario(PathBuf);

pub type GameRng = bevy_rand::prelude::WyRand;

impl Game {
    pub fn new(scenario_path: impl Into<PathBuf>, time_step: Duration) -> Self {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(LogPlugin::default());
        app.add_plugins(TransformPlugin);
        app.insert_resource(Time::<Fixed>::from_duration(time_step));
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(EntropyPlugin::<GameRng>::with_seed([0; 8]));
        app.add_plugins(plugins::plugin);

        app.init_resource::<Time>();
        app.init_resource::<PrimitiveCache>();
        app.init_resource::<SpawnLookup>();
        app.insert_resource(SensitivityConfig(schema::SensitivityConfig {
            sensitivity: 1.0,
            sensitivity_factor: 0.022,
        }));

        app.add_observer(load_scenario);

        app.world_mut().trigger(LoadScenario(scenario_path.into()));

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

    pub fn shapes(&self, mut f: impl FnMut(Transform, Shape)) {
        let mut query = self
            .app
            .world()
            .try_query_filtered::<(&GlobalTransform, &Shape), ()>()
            .unwrap();

        for (transform, shape) in query.iter(self.app.world()) {
            f((*transform).into(), *shape);
        }
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct Input {
    pub look: Vec2,
    pub movement: Vec3,
}

fn load_scenario(
    load_scenario: On<LoadScenario>,
    mut commands: Commands,
    mut spawn_lookup: ResMut<SpawnLookup>,
) -> Result {
    let (scenario, _): (schema::Scenario, _) = ref_asset::io::read_file(&load_scenario.0)?;

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

        let (anchor, view) = if let Some(eyes) = character.eyes {
            let anchor_transform = Transform::from_xyz(0.0, -shape.extents().y + eyes.height, 0.0);
            let view_transform = Transform::from_xyz(0.0, 0.0, eyes.offset);

            let anchor = commands.spawn((anchor_transform, ChildOf(entity))).id();
            let view = commands.spawn((view_transform, ChildOf(anchor))).id();

            (anchor, view)
        } else {
            (entity, entity)
        };

        commands.entity(anchor).insert(FacingOf(entity));

        view
    }

    commands
        .spawn(Spawner {
            spawn_rules: SpawnRules { limit: 1 },
            spawn_group: SpawnGroup(0),
        })
        .observe(move |spawned: On<Spawned>, mut commands: Commands| {
            let view = character(
                commands.reborrow(),
                spawned.spawned,
                scenario.player.character,
                scenario.player.movement,
            );

            commands.entity(spawned.spawned).insert((
                Player,
                InputDriver,
                CollisionLayers::character(Team::PLAYER),
            ));

            commands.entity(view).insert(Camera);
        });

    if let Some(bot_template) = scenario.bot_template {
        commands
            .spawn(Spawner {
                spawn_rules: bot_template.spawn_rules,
                spawn_group: SpawnGroup(1),
            })
            .observe(move |spawned: On<Spawned>, mut commands: Commands| {
                character(
                    commands.reborrow(),
                    spawned.spawned,
                    scenario.player.character,
                    scenario.player.movement,
                );

                commands.entity(spawned.spawned).insert((
                    Bot,
                    AutoDriver::from(bot_template.driver.clone()),
                    CollisionLayers::character(Team::BOT),
                ));
            });
    }

    Ok(())
}
