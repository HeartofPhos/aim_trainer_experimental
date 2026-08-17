use glam::prelude::*;
use glamx::Pose3;
use rand::Rng;
use schema::{BrushTransform, SpawnDef, SpawnGroup, SpawnRules};
use std::collections::HashMap;

use crate::game::weighted_random;

#[derive(Default)]
pub struct SpawnLookup {
    spawns: HashMap<SpawnGroup, WeightedSpawn>,
}

#[derive(Default)]
pub struct WeightedSpawn {
    total_weight: f32,
    transforms: Vec<(f32, Mat4)>,
}

impl SpawnLookup {
    pub fn push(&mut self, def: &SpawnDef, transform: BrushTransform) {
        let scale = transform.bounds.extents();
        let rotation = transform.facing;
        let translation = transform.bounds.center();

        let weight = Vec3::dot(scale, scale);

        let spawn = self.spawns.entry(def.group).or_default();

        spawn.total_weight += weight;
        spawn.transforms.push((
            weight,
            Mat4::from_scale_rotation_translation(scale, rotation, translation),
        ));
    }

    pub fn get_spawn(
        &self,
        group: SpawnGroup,
        target_extents: Vec3,
        rng: &mut impl Rng,
    ) -> Option<(Vec3, Quat)> {
        let weighted_spawn = self.spawns.get(&group)?;

        let spawn_to_world = weighted_random(
            weighted_spawn.total_weight,
            weighted_spawn.transforms.iter().map(|x| (x.0, x.1)),
            rng,
        )?;

        let target_to_spawn = spawn_to_world.inverse();

        let extents = target_to_spawn.transform_vector3(target_extents).abs();
        let spawn_extents = Vec3::splat(0.5);

        let spawn_min = Vec3::min(extents - spawn_extents, Vec3::ZERO);
        let spawn_max = Vec3::max(spawn_extents - extents, Vec3::ZERO);

        let mut delta = spawn_max - spawn_min;

        delta.x *= rand::random::<f32>();
        delta.y *= rand::random::<f32>();
        delta.z *= rand::random::<f32>();

        let translation = spawn_to_world.transform_point3(spawn_min + delta);
        let rotation = spawn_to_world.to_scale_rotation_translation().1;

        Some((translation, rotation))
    }
}

pub struct Spawner {
    pub spawn_rules: SpawnRules,
    pub spawn_group: SpawnGroup,
    pub spawned: Vec<Pose3>,
}

pub fn update_spawn(spawner: &mut Spawner, spawn_lookup: &SpawnLookup, rng: &mut impl Rng) {
    for _ in spawner.spawned.len()..=spawner.spawn_rules.limit {
        let (translation, rotation) = spawn_lookup
            .get_spawn(spawner.spawn_group, Vec3::ZERO, rng)
            .unwrap_or_default();

        spawner.spawned.push(Pose3 {
            rotation,
            translation,
            padding: Default::default(),
        });
    }
}
