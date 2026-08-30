use crate::{
    GameRng, Transform,
    logic::{SpawnSet, shape::ShapeExtents},
    utils::weighted_random,
};
use bevy::prelude::*;
use bevy_rand::prelude::*;
use rand::RngExt;
use schema::{BrushTransform, SpawnGroup, SpawnRules};
use std::collections::HashMap;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, spawner.in_set(SpawnSet::Spawn));
    app.add_systems(FixedUpdate, move_to_spawn.in_set(SpawnSet::Move));
}

crate::relationships! {
    crate::relationship!(pub, one_many, SpawnedBy, [], SpawnedList, []);
}

#[derive(Resource, Default)]
pub struct SpawnLookup {
    spawns: HashMap<SpawnGroup, WeightedSpawn>,
}

#[derive(Default)]
pub struct WeightedSpawn {
    total_weight: f32,
    transforms: Vec<(f32, Mat4)>,
}

impl SpawnLookup {
    pub fn push(&mut self, spawn_group: SpawnGroup, transform: BrushTransform) {
        let weight = Vec3::dot(transform.scale, transform.scale);

        let spawn = self.spawns.entry(spawn_group).or_default();

        spawn.total_weight += weight;
        spawn.transforms.push((
            weight,
            Mat4::from_scale_rotation_translation(
                transform.scale,
                transform.rotation,
                transform.translation,
            ),
        ));
    }

    pub fn get_spawn(
        &self,
        group: SpawnGroup,
        target_extents: Vec3,
        rng: &mut GameRng,
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

        delta.x *= rng.random::<f32>();
        delta.y *= rng.random::<f32>();
        delta.z *= rng.random::<f32>();

        let translation = spawn_to_world.transform_point3(spawn_min + delta);
        let rotation = spawn_to_world.to_scale_rotation_translation().1;

        Some((translation, rotation))
    }
}

#[derive(Component)]
pub struct Spawner {
    pub spawn_rules: SpawnRules,
    pub spawn_group: SpawnGroup,
}

#[derive(Component)]
struct MoveToSpawn;

#[derive(EntityEvent)]
pub struct Spawned {
    #[event_target]
    pub spawner: Entity,
    pub spawned: Entity,
}

fn spawner(
    mut commands: Commands,
    query: Query<(Entity, &Spawner, Option<&SpawnedList>)>,
    mut global: Single<&mut GameRng, With<GlobalRng>>,
) {
    for (spawner_entity, spawner, spawned_list) in query {
        let spawned_count = spawned_list.map(|x| x.entities().len()).unwrap_or(0);
        for _ in spawned_count..spawner.spawn_rules.limit {
            let spawned_entity = commands
                .spawn((SpawnedBy(spawner_entity), MoveToSpawn, global.fork_seed()))
                .id();

            commands.trigger(Spawned {
                spawner: spawner_entity,
                spawned: spawned_entity,
            });
        }
    }
}

fn move_to_spawn(
    mut commands: Commands,
    spawn_lookup: Res<SpawnLookup>,
    spawner_query: Query<&Spawner>,
    query: Query<(Entity, &SpawnedBy, Option<&ShapeExtents>, &mut GameRng), With<MoveToSpawn>>,
) -> Result {
    for (entity, spawned_by, shape_extents, mut rng) in query {
        let extents = match shape_extents {
            Some(shape_extents) => shape_extents.0,
            None => {
                warn!(?entity, "missing shape extents");
                Vec3::ZERO
            }
        };

        let spawner = spawner_query.get(spawned_by.0)?;
        let (translation, rotation) = spawn_lookup
            .get_spawn(spawner.spawn_group, extents, &mut rng)
            .unwrap_or_default();

        commands
            .entity(entity)
            .insert(Transform {
                translation,
                rotation,
                ..Default::default()
            })
            .remove::<MoveToSpawn>();
    }

    Ok(())
}
