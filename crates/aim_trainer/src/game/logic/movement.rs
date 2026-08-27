use crate::game::{
    Transform,
    logic::{MovementSet, TimeFactor, character_controller::Grounded},
    utils::Direction,
};
use avian3d::prelude::*;
use bevy::{ecs::system::SystemParam, math::FloatPow, prelude::*};
use derive_more::Deref;
use schema::MovementMode;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            jump, impulses, grounded, rotate, friction, accelerate, gravity,
        )
            .chain()
            .in_set(MovementSet),
    );
}

crate::relationships! {
    pub
    crate::relationship!(pub, one_one, FacingOf, [], Facing, []);
}

#[derive(Component, Default, Clone, Copy, Deref)]
#[component(immutable)]
#[require(Transform, MovementInput)]
pub struct MovementProfile(pub schema::MovementProfile);

impl MovementProfile {
    pub fn get_excluded_axes(&self) -> impl Iterator<Item = Vec3> {
        let excluded_axes = match self.mode {
            MovementMode::Fly => None,
            MovementMode::Jump { .. } => Some(Vec3::UP),
        };

        excluded_axes.into_iter()
    }
}

#[derive(Component, Default)]
pub struct MovementInput {
    pub direction: Option<Dir3>,
    pub impulses: Vec<(Dir3, f32)>,
    pub pitch_delta: f32,
    pub yaw_delta: f32,
}

fn apply_impulse(transform: &Transform, mut velocity: Vec3, dir: Dir3, speed: f32) -> Vec3 {
    let dir = transform.rotation * dir.as_vec3();
    let [dir_velocity] = exclude_axes([dir], [velocity]);

    velocity -= dir_velocity;
    let current_speed = Vec3::dot(dir_velocity, dir);
    let new_speed = f32::max(speed, current_speed);
    velocity += dir * new_speed;

    velocity
}

pub fn exclude_axes<const N: usize>(
    excluded_axes: impl IntoIterator<Item = Vec3>,
    mut values: [Vec3; N],
) -> [Vec3; N] {
    let mut output = [Vec3::ZERO; N];

    for axis in excluded_axes {
        for i in 0..N {
            let proj = values[i].project_onto_normalized(axis);
            values[i] -= proj;
            output[i] += proj;
        }
    }

    output
}

#[derive(SystemParam)]
pub struct SpeedThreshold<'w, 's> {
    query: Query<
        'w,
        's,
        (
            &'static MovementInput,
            &'static LinearVelocity,
            &'static MovementProfile,
            &'static Facing,
        ),
    >,
    transform_query: Query<'w, 's, &'static GlobalTransform>,
}

impl<'w, 's> SpeedThreshold<'w, 's> {
    pub fn pass_speed_threshold(&self, entity: Entity, threshold: f32) -> Result<bool> {
        let (mi, lin_vel, mp, facing) = self.query.get(entity)?;
        let facing = self.transform_query.get(facing.entity())?;

        let speed_squared = match mi.direction {
            Some(direction) => {
                Vec3::dot(facing.rotation() * direction.as_vec3(), lin_vel.0).squared()
            }
            None => lin_vel.0.length_squared(),
        };

        Ok(speed_squared >= (mp.max_speed * threshold).squared())
    }
}

const PITCH_LIMIT: f32 = 89.00 * (std::f32::consts::PI / 180.0);
fn rotate(
    query: Query<(&MovementInput, &Facing)>,
    mut facing_query: Query<&mut Transform>,
) -> Result {
    for (mi, facing) in query {
        let mut facing = facing_query.get_mut(facing.entity())?;

        let (yaw, pitch, roll) = facing.rotation.to_euler(EulerRot::YXZ);

        let pitch_wish = f32::clamp(pitch + mi.pitch_delta, -PITCH_LIMIT, PITCH_LIMIT);
        let yaw_wish = yaw + mi.yaw_delta;

        facing.rotation = Quat::from_euler(EulerRot::YXZ, yaw_wish, pitch_wish, roll);
    }

    Ok(())
}

fn jump(query: Query<(&mut MovementInput, &MovementProfile), With<Grounded>>) {
    for (mut mi, mp) in query {
        if let MovementMode::Jump { speed } = mp.mode
            && matches!(mi.direction, Some(dir) if dir.y > 0.0)
        {
            mi.impulses.push((Dir3::UP, speed));
        }
    }
}

fn impulses(query: Query<(&mut MovementInput, &mut LinearVelocity, &Transform)>) {
    for (mut mi, mut lin_vel, transform) in query {
        for impulse in &mi.impulses {
            lin_vel.0 = apply_impulse(transform, lin_vel.0, impulse.0, impulse.1);
        }
        mi.impulses.clear();
    }
}

fn grounded(
    mut commands: Commands,
    query: Query<(Entity, &LinearVelocity, &Transform), With<Grounded>>,
) {
    for (entity, lin_vel, transform) in query {
        const UNGROUND_MIN_SPEED: f32 = 0.01;
        if Vec3::dot(lin_vel.0, transform.rotation * Vec3::UP) > UNGROUND_MIN_SPEED {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn friction(
    time: Res<Time>,
    query: Query<(
        Has<Grounded>,
        &MovementProfile,
        &mut LinearVelocity,
        &TimeFactor,
    )>,
) {
    for (grounded, mp, mut lin_vel, time_factor) in query {
        let friction = if grounded {
            mp.friction
        } else {
            mp.air_friction
        };

        pm_friction(
            &mut lin_vel.0,
            friction,
            mp.stop_speed,
            time.delta_secs() * time_factor.0,
        );
    }
}

fn accelerate(
    time: Res<Time>,
    query: Query<(
        Has<Grounded>,
        &MovementInput,
        &MovementProfile,
        &mut LinearVelocity,
        &Facing,
        &Transform,
        &TimeFactor,
    )>,
    facing_query: Query<&Transform>,
) -> Result {
    for (grounded, mi, mp, mut lin_vel, facing, transform, time_factor) in query {
        let facing = facing_query.get(facing.entity())?;

        let accel = if grounded {
            mp.accelerate
        } else {
            mp.air_accelerate
        };

        let accelerate_fn = if mp.race {
            pm_accelerate_race
        } else {
            pm_accelerate
        };

        let mut wish_dir: Vec3 = mi.direction.map(Into::into).unwrap_or_default();
        let [excluded_wish_dir] = exclude_axes(mp.get_excluded_axes(), [wish_dir]);
        wish_dir -= excluded_wish_dir;

        let mut local_wish_dir = facing.rotation * wish_dir;
        let mut local_velocity = transform.rotation.inverse() * lin_vel.0;

        let [excluded_wish_dir, excluded_velocity] =
            exclude_axes(mp.get_excluded_axes(), [local_wish_dir, local_velocity]);

        local_wish_dir -= excluded_wish_dir;
        local_velocity -= excluded_velocity;

        accelerate_fn(
            &mut local_velocity,
            local_wish_dir.normalize_or_zero(),
            accel,
            mp.max_speed,
            time.delta_secs() * time_factor.0,
        );

        local_velocity += excluded_velocity;

        lin_vel.0 = transform.rotation * local_velocity;
    }

    Ok(())
}

fn gravity(
    time: Res<Time>,
    query: Query<
        (
            &Transform,
            &MovementProfile,
            &mut LinearVelocity,
            &TimeFactor,
        ),
        Without<Grounded>,
    >,
) {
    for (transform, mp, mut lin_vel, time_factor) in query {
        let up = transform.rotation * Vec3::UP;
        lin_vel.0 += up * mp.gravity * time.delta_secs() * time_factor.0;
    }
}

fn pm_friction(velocity: &mut Vec3, friction: f32, stop_speed: f32, dt: f32) {
    let speed = velocity.length();
    if speed == 0.0 {
        return;
    }

    let control = f32::max(speed, stop_speed);
    let drop = control * friction * dt;

    let new_speed = f32::max(speed - drop, 0.0) / speed;

    *velocity *= new_speed;
}

fn pm_accelerate_race(velocity: &mut Vec3, wish_dir: Vec3, accel: f32, wish_speed: f32, dt: f32) {
    let current_speed = Vec3::dot(*velocity, wish_dir);
    let add_speed = wish_speed - current_speed;
    if add_speed <= 0.0 {
        return;
    }

    let accel_speed = f32::min(accel * dt * wish_speed, add_speed);

    *velocity += accel_speed * wish_dir;
}

fn pm_accelerate(velocity: &mut Vec3, wish_dir: Vec3, accel: f32, wish_speed: f32, dt: f32) {
    if wish_dir == Vec3::ZERO {
        return;
    }

    let wish_velocity = wish_dir * wish_speed;
    let mut push_dir = wish_velocity - *velocity;

    let push_len = push_dir.length();
    push_dir = if push_len.is_finite() && push_len > 0.0 {
        push_dir * push_len.recip()
    } else {
        Vec3::ZERO
    };

    let can_push = f32::min(accel * dt * wish_speed, push_len);

    *velocity += push_dir * can_push;
}
