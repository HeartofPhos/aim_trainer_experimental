use crate::game::{
    config::SensitivityConfig,
    level::{BrushDef, BrushTransform},
    plugins::{
        input_driver::InputDriver,
        movement::{Facing, MovementProfile},
        spawn::{SpawnLookup, Spawned, Spawner},
    },
};
use bevy::{prelude::*, time::TimeUpdateStrategy};
use bevy_rand::plugin::EntropyPlugin;
use schema::{SpawnGroup, SpawnRules};
use std::{path::PathBuf, time::Duration};

mod config;
mod level;
mod plugins;
mod utils;

pub struct Game {
    app: App,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Bot;

#[derive(Event)]
struct LoadScenario(PathBuf);

pub type GameRng = bevy_rand::prelude::WyRand;

impl Game {
    pub fn new(scenario_path: impl Into<PathBuf>) -> Self {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(EntropyPlugin::<GameRng>::with_seed([0; 8]));
        app.add_plugins(plugins::plugin);

        app.init_resource::<Time>();
        app.init_resource::<SpawnLookup>();

        app.add_observer(load_scenario);

        app.world_mut().trigger(LoadScenario(scenario_path.into()));

        Self { app }
    }

    pub fn level_brushes(&self, mut f: impl FnMut(&schema::BrushDef, &schema::BrushTransform)) {
        for (brush_def, brush_transform) in self
            .app
            .world()
            .try_query::<(&BrushDef, &BrushTransform)>()
            .unwrap()
            .iter(self.app.world())
        {
            f(brush_def, brush_transform);
        }
    }

    pub fn update(&mut self, delta_time: Duration, input: Input) {
        let mut time = self.app.world_mut().resource_mut::<TimeUpdateStrategy>();
        *time = TimeUpdateStrategy::ManualDuration(delta_time);

        self.app.insert_resource(input);

        self.app.update();
    }

    pub fn player(&self) -> Result<Transform> {
        let (transform, facing) = self
            .app
            .world()
            .try_query_filtered::<(&Transform, &Facing), With<Player>>()
            .ok_or("failed query")?
            .single(self.app.world())?;

        Ok(Transform {
            translation: transform.translation,
            rotation: transform.rotation * facing.0,
            ..Default::default()
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
