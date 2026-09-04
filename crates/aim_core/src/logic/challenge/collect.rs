use crate::logic::{
    challenge::{ChallengeSet, buffer::BufferMultiplier},
    character::movement::SpeedThreshold,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use core::f32;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (is_moving, in_range, write)
            .chain()
            .in_set(ChallengeSet::Accumulate),
    );

    app.add_observer(on_add_collectable_sensor);
}

#[derive(Component)]
#[require(Sensor)]
pub struct CollectableSensor;

#[derive(Component, Default, Clone, Copy)]
#[require(CollectState)]
pub struct CollectProfile(pub schema::CollectProfile);

#[derive(Component)]
struct CollectState {
    multiplier: f32,
    is_moving: bool,
    interesect_count: u32,
}

impl Default for CollectState {
    fn default() -> Self {
        Self {
            multiplier: 1.0,
            is_moving: false,
            interesect_count: 0,
        }
    }
}

const COLLECT_MIN: f32 = 0.0;
const COLLECT_MAX: f32 = 1.0;
const COLLECT_FILL: f32 = COLLECT_MAX - COLLECT_MIN;
const COLLECT_DRAIN: f32 = COLLECT_MIN - COLLECT_MAX;

fn on_add_collectable_sensor(add: On<Add, CollectableSensor>, mut commands: Commands) {
    commands
        .entity(add.entity)
        .observe(on_collision_start)
        .observe(on_collision_end);
}

fn on_collision_start(
    collision: On<CollisionStart>,
    mut query: Query<&mut CollectState>,
) -> Result {
    let mut state = query.get_mut(collision.collider2).ignore()?;
    state.interesect_count += 1;
    Ok(())
}

fn on_collision_end(collision: On<CollisionEnd>, mut query: Query<&mut CollectState>) -> Result {
    let mut state = query.get_mut(collision.collider2).ignore()?;
    state.interesect_count -= 1;
    Ok(())
}

fn write(query: Query<(Entity, &CollectState)>, mut commands: Commands) {
    for (entity, state) in query {
        commands
            .entity(entity)
            .insert(BufferMultiplier(state.multiplier));
    }
}

fn is_moving(
    mut query: Query<(Entity, &mut CollectState, &CollectProfile)>,
    speed_threshold: SpeedThreshold,
) -> Result {
    for (entity, mut collect_state, collect_profile) in &mut query {
        collect_state.is_moving =
            speed_threshold.pass_speed_threshold(entity, collect_profile.0.speed_threshold)?;
    }

    Ok(())
}

fn in_range(time: Res<Time>, query: Query<(&mut CollectState, &CollectProfile)>) -> Result {
    for (mut collect_state, collect_profile) in query {
        let collecting = collect_state.is_moving && collect_state.interesect_count > 0;

        let rate = if collecting {
            COLLECT_FILL / collect_profile.0.fill_time.as_secs_f32()
        } else {
            COLLECT_DRAIN / collect_profile.0.drain_time.as_secs_f32()
        };

        collect_state.multiplier += rate * time.delta().as_secs_f32();
        collect_state.multiplier = f32::clamp(collect_state.multiplier, COLLECT_MIN, COLLECT_MAX);
    }

    Ok(())
}
