use crate::game::spawn::{SpawnLookup, Spawner, update_spawn};
use glam::prelude::*;
use glamx::prelude::*;
use rand::{Rng, RngExt, SeedableRng, rngs::Xoshiro128PlusPlus};
use schema::{Brush, Scenario, SpawnGroup, SpawnRules};
use std::{path::Path, time::Duration};

mod spawn;

pub struct Game {
    scenario: Scenario,
    spawn_lookup: SpawnLookup,
    player_spawner: Spawner,
    rng: Xoshiro128PlusPlus,
}

impl Game {
    pub fn new(scenario_path: impl AsRef<Path>) -> Self {
        let (scenario, _): (schema::Scenario, _) = ref_asset::io::read_file(scenario_path).unwrap();

        let mut rng = Xoshiro128PlusPlus::seed_from_u64(0);

        let mut spawn_lookup = SpawnLookup::default();
        for brush in &scenario.level.brush_list {
            match &brush.def {
                schema::BrushDef::Spawn(spawn_def) => {
                    spawn_lookup.push(spawn_def, brush.transform);
                }
                schema::BrushDef::Primitive(_) => (),
            }
        }

        Self {
            scenario,
            spawn_lookup,
            player_spawner: Spawner {
                spawn_rules: SpawnRules { limit: 1 },
                spawn_group: SpawnGroup(0),
                spawned: Default::default(),
            },
            rng,
        }
    }

    pub fn level_brushes(&self) -> &[Brush] {
        &self.scenario.level.brush_list
    }

    pub fn update(&mut self, delta_time: Duration, input: Input) {
        update_spawn(&mut self.player_spawner, &self.spawn_lookup, &mut self.rng);

        let player = &mut self.player_spawner.spawned[0];

        const PITCH_LIMIT: f32 = 89.00 * (std::f32::consts::PI / 180.0);

        let pitch_delta = f32::to_radians(-input.look.y) * 0.022;
        let yaw_delta = f32::to_radians(-input.look.x) * 0.022;

        let (yaw, pitch, roll) = player.rotation.to_euler(EulerRot::YXZ);

        let pitch_wish = f32::clamp(pitch + pitch_delta, -PITCH_LIMIT, PITCH_LIMIT);
        let yaw_wish = yaw + yaw_delta;

        player.rotation = Quat::from_euler(EulerRot::YXZ, yaw_wish, pitch_wish, roll);

        let dir = player.rotation * input.movement_dir;
        player.translation +=
            dir * self.scenario.player.movement.max_speed * delta_time.as_secs_f32();
    }

    pub fn player(&self) -> Option<Pose3> {
        self.player_spawner.spawned.first().copied()
    }
}

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub look: Vec2,
    pub movement_dir: Vec3,
}

// TODO https://www.keithschwarz.com/darts-dice-coins/
fn weighted_random<T>(
    total_weight: f32,
    iter: impl IntoIterator<Item = (f32, T)>,
    rng: &mut impl Rng,
) -> Option<T> {
    let mut random_weight = rng.random::<f32>() * total_weight;

    for (weight, value) in iter {
        if random_weight < weight {
            return Some(value);
        }

        random_weight -= weight;
    }

    None
}
