use crate::game::{
    config::SensitivityConfig,
    level::{BrushDef, BrushTransform},
    plugins::{
        input_driver::InputDriver,
        movement::{Facing, MovementProfile},
        spawn::{SpawnLookup, Spawned, Spawner},
    },
    random::Random,
};
use bevy_ecs::{prelude::*, schedule::ScheduleLabel};
use bevy_math::prelude::*;
use rand::SeedableRng;
use schema::{SpawnGroup, SpawnRules};
use std::{path::PathBuf, time::Duration};

mod config;
mod level;
mod plugins;
mod random;
mod utils;

pub struct Game {
    world: World,
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash, Default)]
struct Update;

#[derive(Component, Default, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
}

#[derive(Resource, Default)]
struct Time {
    elapsed: Duration,
    delta_time: Duration,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Bot;

#[derive(Event)]
struct LoadScenario(PathBuf);

impl Game {
    pub fn new(scenario_path: impl Into<PathBuf>) -> Self {
        let mut world = World::new();

        world.init_resource::<Time>();
        world.init_resource::<SpawnLookup>();
        world.insert_resource(Random::seed_from_u64(0));

        world.add_observer(load_scenario);

        plugins::plugin(&mut world);

        world.trigger(LoadScenario(scenario_path.into()));

        Self { world }
    }

    pub fn level_brushes(&self, mut f: impl FnMut(&schema::BrushDef, &schema::BrushTransform)) {
        for (brush_def, brush_transform) in self
            .world
            .try_query::<(&BrushDef, &BrushTransform)>()
            .unwrap()
            .iter(&self.world)
        {
            f(brush_def, brush_transform);
        }
    }

    pub fn update(&mut self, delta_time: Duration, input: Input) {
        let mut time = self.world.resource_mut::<Time>();
        time.elapsed += delta_time;
        time.delta_time = delta_time;
        self.world.insert_resource(input);

        self.world.run_schedule(Update);
    }

    pub fn player(&self) -> Result<Transform> {
        let (transform, facing) = self
            .world
            .try_query_filtered::<(&Transform, &Facing), With<Player>>()
            .ok_or("failed query")?
            .single(&self.world)?;

        Ok(Transform {
            translation: transform.translation,
            rotation: transform.rotation * facing.0,
        })
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

    commands.insert_resource(SensitivityConfig(schema::SensitivityConfig {
        sensitivity: 1.0,
        sensitivity_factor: 0.022,
    }));

    for brush in scenario.level.brush_list {
        match brush.def {
            schema::BrushDef::Spawn { group } => {
                spawn_lookup.push(group, brush.transform);
            }
            schema::BrushDef::Primitive { .. } => (),
        }

        commands.spawn((BrushTransform(brush.transform), BrushDef(brush.def)));
    }

    commands
        .spawn(Spawner {
            spawn_rules: SpawnRules { limit: 1 },
            spawn_group: SpawnGroup(0),
        })
        .observe(move |spawned: On<Spawned>, mut commands: Commands| {
            commands.entity(spawned.spawned).insert((
                Player,
                InputDriver,
                MovementProfile(scenario.player.movement),
            ));
        });

    if let Some(bot_template) = scenario.bot_template {
        commands
            .spawn(Spawner {
                spawn_rules: bot_template.spawn_rules,
                spawn_group: SpawnGroup(1),
            })
            .observe(|spawned: On<Spawned>, mut commands: Commands| {
                commands.entity(spawned.spawned).insert(Bot);
            });
    }

    Ok(())
}
