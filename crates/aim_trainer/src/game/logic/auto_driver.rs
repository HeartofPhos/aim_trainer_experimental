use crate::game::{
    GameRng, Player,
    logic::{
        DriverSet, TimeFactor,
        movement::{Facing, MovementInput, MovementProfile, SpeedThreshold, exclude_axes},
    },
    utils::{Direction, weighted_random},
};
use avian3d::prelude::*;
use bevy::{math::FloatPow, prelude::*};
use rand::RngExt;
use schema::{
    AutoDriverProfile, Follow, FollowTarget, MagnitudeVector, UnitVector, Unstick, Variant,
};
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            target,
            facing,
            unstick,
            dodge,
            impulse,
            follow,
            consume_input,
        )
            .chain()
            .in_set(DriverSet),
    );
}

crate::relationships! {
    crate::relationship!(pub, one_many, AutoDriverTarget, [], AutoDriverTargetOf, []);
}

#[derive(Component, Default, Clone)]
#[component(immutable)]
#[require(AutoDriverState, NetInput)]
pub struct AutoDriver {
    follow: Option<Follow>,
    unstick: Option<Unstick>,
    dodge_layers: Vec<DriverLayer<UnitVector>>,
    impulse_layers: Vec<DriverLayer<MagnitudeVector>>,
}

#[derive(Component, Default)]
struct NetInput(Vec3);

impl From<AutoDriverProfile> for AutoDriver {
    fn from(value: AutoDriverProfile) -> Self {
        Self {
            follow: value.follow,
            unstick: value.unstick,
            dodge_layers: value.dodge_layers.into_iter().map(Into::into).collect(),
            impulse_layers: value.impulse_layers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone)]
struct DriverLayer<T> {
    total_weight: f32,
    variants: Vec<Variant<T>>,
}

impl<T> DriverLayer<T> {
    fn get_random(&self, rng: &mut impl RngExt) -> Option<&Variant<T>> {
        weighted_random(
            self.total_weight,
            self.variants.iter().map(|x| (x.weight, x)),
            rng,
        )
    }
}

impl<T> From<schema::DriverLayer<T>> for DriverLayer<T> {
    fn from(value: schema::DriverLayer<T>) -> Self {
        DriverLayer {
            total_weight: value.0.iter().map(|x| x.weight).sum(),
            variants: value.0,
        }
    }
}

trait Sample {
    type Output;
    fn sample(&self, rng: &mut impl RngExt) -> Self::Output;
}

impl Sample for UnitVector {
    type Output = Option<Dir3>;

    fn sample(&self, rng: &mut impl RngExt) -> Self::Output {
        match self {
            Self::Value(value) => (*value).try_into().ok(),
            Self::Circle(normal) => {
                let rotation = Quat::from_rotation_arc(Vec3::FORWARD, *normal);
                let dir = rotation * Circle::new(1.0).sample_boundary(rng).extend(0.0);
                Some(dir.try_into().expect("invalid dir"))
            }
            Self::Sphere => {
                let dir = Sphere::new(1.0).sample_boundary(rng);
                Some(dir.try_into().expect("invalid dir"))
            }
        }
    }
}

impl Sample for MagnitudeVector {
    type Output = Option<(Dir3, f32)>;

    fn sample(&self, rng: &mut impl RngExt) -> Self::Output {
        let MagnitudeVector(unit, speed) = self;

        unit.sample(rng).map(|unit| (unit, *speed))
    }
}

#[derive(Component, Default)]
struct AutoDriverState {
    pub dodge_state: Vec<Option<DodgeState>>,
    pub impulse_state: Vec<Option<ImpulseState>>,
    pub unstick_timer: Option<Timer>,
}

#[derive(Clone)]
struct DodgeState {
    dir: Option<Dir3>,
    timer: Timer,
}

#[derive(Clone)]
struct ImpulseState {
    dir: Dir3,
    speed: f32,
    timer: Timer,
}

fn random_timer(min: f32, max: f32, rng: &mut impl RngExt) -> Timer {
    let range = min..max;
    let new_duration = if range.is_empty() {
        range.start
    } else {
        rng.random_range(min..max)
    };

    let new_duration = Duration::from_secs_f32(new_duration);
    Timer::new(new_duration, TimerMode::Once)
}

fn consume_input(query: Query<(&mut NetInput, &mut MovementInput)>) {
    for (mut net_input, mut movement_input) in query {
        movement_input.direction = Dir3::new(net_input.0).ok();
        net_input.0 = Vec3::ZERO;
    }
}

fn unstick(
    time: Res<Time>,
    query: Query<(
        Entity,
        &MovementInput,
        &MovementProfile,
        &mut AutoDriverState,
        &AutoDriver,
        &Facing,
        &Collider,
        &CollisionLayers,
        &TimeFactor,
        &mut GameRng,
    )>,
    speed_threshold: SpeedThreshold,
    transform_query: Query<&GlobalTransform>,
    move_and_slide: MoveAndSlide,
) -> Result {
    for (
        entity,
        mi,
        mp,
        mut driver_state,
        driver,
        facing,
        collider,
        collision_layers,
        time_factor,
        mut rng,
    ) in query
    {
        if let Some(unstick) = driver.unstick {
            let unstick_timer = driver_state
                .unstick_timer
                .get_or_insert_with(|| random_timer(unstick.min, unstick.max, &mut rng));

            if speed_threshold.pass_speed_threshold(entity, unstick.speed_threshold)? {
                unstick_timer.reset();
                continue;
            }

            unstick_timer.tick(time.delta().mul_f32(time_factor.0));

            if !unstick_timer.just_finished() {
                continue;
            }

            driver_state.unstick_timer = None;

            let Some(input_dir) = mi.direction else {
                continue;
            };

            let facing = transform_query.get(facing.entity())?;

            let mut wish_dir = input_dir.as_vec3();
            let [excluded_wish_dir] = exclude_axes(mp.get_excluded_axes(), [wish_dir]);
            wish_dir -= excluded_wish_dir;
            wish_dir = facing.rotation() * wish_dir;

            let Ok(wish_dir) = wish_dir.try_into() else {
                continue;
            };

            let global_transform = transform_query.get(entity)?;

            let translation = global_transform.translation();
            let rotation = global_transform.rotation();

            const UNSTICK_MAX_DISTANCE: f32 = 0.1;

            let hit = move_and_slide.spatial_query.cast_shape_predicate(
                collider,
                translation,
                rotation,
                wish_dir,
                &ShapeCastConfig::from_max_distance(UNSTICK_MAX_DISTANCE),
                &SpatialQueryFilter::from_collision_layers(*collision_layers)
                    .with_excluded_entities([entity]),
                // Make sure we don't hit sensors.
                // TODO: Replace this when spatial queries support excluding sensors directly.
                &|entity| move_and_slide.colliders.contains(entity),
            );

            let Some(hit) = hit else {
                continue;
            };

            let mut normal = facing.rotation().inverse() * hit.normal1;

            let [excluded_normal] = exclude_axes(mp.get_excluded_axes(), [normal]);
            normal -= excluded_normal;

            let Some(normal) = normal.try_normalize() else {
                continue;
            };

            for dodge_state in driver_state.dodge_state.iter_mut().flatten() {
                if let Some(dir) = &mut dodge_state.dir {
                    *dir = dir
                        .reflect(normal)
                        .try_into()
                        .expect("invalid dir reflection");
                }
            }
        }
    }

    Ok(())
}

fn dodge(
    time: Res<Time>,
    query: Query<(
        &mut AutoDriverState,
        &AutoDriver,
        &mut NetInput,
        &TimeFactor,
        &mut GameRng,
    )>,
) -> Result {
    for (mut driver_state, driver, mut net_input, time_factor, mut rng) in query {
        let layers = &driver.dodge_layers;
        let state = &mut driver_state.dodge_state;

        if state.len() != layers.len() {
            *state = vec![None; layers.len()];
        }

        for (layer, state) in std::iter::zip(layers.iter(), state.iter_mut()) {
            if let Some(value) = state {
                value.timer.tick(time.delta().mul_f32(time_factor.0));
                if value.timer.just_finished() {
                    *state = None;
                }
            }

            if state.is_none()
                && let Some(variant) = layer.get_random(&mut rng)
            {
                *state = Some(DodgeState {
                    dir: variant.value.sample(&mut rng),
                    timer: random_timer(variant.min, variant.max, &mut rng),
                });
            }
        }

        for dodge_state in driver_state.dodge_state.iter().flatten() {
            if let Some(dir) = dodge_state.dir {
                net_input.0 += dir.as_vec3();
            }
        }
    }

    Ok(())
}

fn impulse(
    time: Res<Time>,
    query: Query<(
        &mut AutoDriverState,
        &AutoDriver,
        &mut MovementInput,
        &TimeFactor,
        &mut GameRng,
    )>,
) {
    for (mut driver_state, driver, mut mi, time_factor, mut rng) in query {
        let layers = &driver.impulse_layers;
        let state = &mut driver_state.impulse_state;

        if state.len() != layers.len() {
            *state = vec![None; layers.len()];
        }

        for (layer, state) in std::iter::zip(layers.iter(), state.iter_mut()) {
            if let Some(value) = state {
                value.timer.tick(time.delta().mul_f32(time_factor.0));
                if value.timer.just_finished() {
                    mi.impulses.push((value.dir, value.speed));

                    *state = None;
                }
            }

            if state.is_none()
                && let Some(variant) = layer.get_random(&mut rng)
                && let Some((dir, speed)) = variant.value.sample(&mut rng)
            {
                *state = Some(ImpulseState {
                    dir,
                    speed,
                    timer: random_timer(variant.min, variant.max, &mut rng),
                });
            }
        }
    }
}

fn follow(
    query: Query<(
        Entity,
        &AutoDriver,
        Option<&AutoDriverTarget>,
        &mut NetInput,
    )>,
    transform_query: Query<&GlobalTransform>,
) -> Result {
    for (entity, driver, driver_target, mut net_input) in query {
        if let (Some(follow), Some(target)) = (driver.follow, driver_target) {
            let target = transform_query.get(target.0)?;
            let transform = transform_query.get(entity)?;

            let target_distance = follow.distance;
            let current_distance =
                (transform.translation() - target.translation()).length_squared();

            let dir = if current_distance > target_distance.max.squared() {
                Vec3::FORWARD
            } else if current_distance < target_distance.min.squared() {
                -Vec3::FORWARD
            } else {
                continue;
            };

            net_input.0 = net_input.0.reject_from_normalized(dir);
            net_input.0 += dir;
        }
    }

    Ok(())
}

fn facing(
    mut driver_query: Query<(Entity, &mut MovementInput, &AutoDriverTarget, &Facing)>,
    target_query: Query<&GlobalTransform>,
) {
    for (entity, mut mi, driver_target, facing) in &mut driver_query {
        if let (Ok(transform), Ok(target), Ok(facing)) = (
            target_query.get(entity),
            target_query.get(driver_target.0),
            target_query.get(facing.entity()),
        ) {
            let wish_facing = (target.translation() - transform.translation()).normalize_or_zero();
            let current_facing = facing.forward();

            let current_yaw = f32::atan2(current_facing.x, current_facing.z);
            let wish_yaw = f32::atan2(wish_facing.x, wish_facing.z);

            let current_pitch = f32::asin(current_facing.y);
            let wish_pitch = f32::asin(wish_facing.y);

            mi.yaw_delta = wish_yaw - current_yaw;
            mi.pitch_delta = wish_pitch - current_pitch;
        }
    }
}

fn target(
    mut commands: Commands,
    driver_query: Query<(Entity, &AutoDriver)>,
    // TODO reservoir sampling?
    player_entity: Single<Entity, With<Player>>,
) {
    for (driver_entity, driver) in &driver_query {
        match driver.follow {
            Some(Follow {
                target: FollowTarget::Player,
                ..
            }) => {
                commands
                    .entity(driver_entity)
                    .insert(AutoDriverTarget(*player_entity));
            }
            None => {
                commands
                    .entity(driver_entity)
                    .try_remove::<AutoDriverTarget>();
            }
        }
    }
}
