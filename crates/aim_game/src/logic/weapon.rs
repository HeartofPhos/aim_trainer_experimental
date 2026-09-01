use crate::{
    Input,
    layers::CollisionLayersExt,
    logic::{WeaponSet, health::Killed, targeter::UseTargeter, team::Team},
    utils::Direction,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use schema::FireMode;
use std::time::Duration;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (input, reload_complete, fire, reload_begin)
            .chain()
            .in_set(WeaponSet),
    );
    app.add_observer(on_shot_fired);
    app.add_observer(on_shot_hit);
    app.add_observer(ammo_on_hit);
    app.add_observer(ammo_on_kill);
}

pub fn weapon_bundle(weapon_profile: schema::WeaponProfile) -> impl Bundle {
    (
        WeaponProfile(weapon_profile),
        WeaponState {
            ammo_current: { weapon_profile.ammo_max },
            ready_to_fire_at: Duration::ZERO,
        },
    )
}

#[derive(Component, Deref, Default, Clone, Copy)]
#[component(immutable)]
#[require(WeaponInput)]
pub struct WeaponProfile(pub schema::WeaponProfile);

#[derive(Component, Default, Clone, Copy)]
pub struct WeaponState {
    ammo_current: u32,
    ready_to_fire_at: Duration,
}

impl WeaponState {
    pub fn ammo_current(&self) -> u32 {
        self.ammo_current
    }
}

#[derive(Component)]
#[component(immutable)]
pub struct Reloading {
    pub started_at: Duration,
    pub completed_at: Duration,
}

#[derive(EntityEvent)]
pub struct ShotFired {
    #[event_target]
    pub shooter: Entity,
    pub shot_info: ShotInfo,
}

#[derive(EntityEvent)]
pub struct ShotHit {
    #[event_target]
    pub target: Entity,
    pub shooter: Entity,
    pub shot_info: ShotInfo,
}

pub enum ShotResult {
    HitTarget(Entity),
    HitObstacle,
    Missed,
}

#[derive(EntityEvent)]
pub struct ShotDone {
    #[event_target]
    pub shooter: Entity,
    pub shot_info: ShotInfo,
    pub result: ShotResult,
}

#[derive(Clone, Copy)]
pub struct ShotInfo {
    pub damage: f32,
    pub interval: Duration,
    pub timestamp: Duration,
}

#[derive(Component, Default)]
struct WeaponInput {
    primary_fire: Option<InputState>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum InputState {
    JustFired,
    Fired,
}

fn input(input: Res<Input>, query: Query<&mut WeaponInput>) -> Result {
    for mut weapon_input in query {
        weapon_input.primary_fire = match (input.fire, weapon_input.primary_fire.is_some()) {
            (true, true) => Some(InputState::Fired),
            (true, false) => Some(InputState::JustFired),
            (false, _) => None,
        };
    }

    Ok(())
}

fn reload_begin(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &WeaponProfile, &WeaponState), Without<Reloading>>,
) {
    for (entity, weapon_profile, weapon_state) in query {
        if weapon_state.ammo_current < weapon_profile.ammo_per_shot {
            commands.entity(entity).insert(Reloading {
                started_at: time.elapsed(),
                completed_at: time.elapsed() + weapon_profile.reload_time,
            });
        }
    }
}

fn reload_complete(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &Reloading, &WeaponProfile, &mut WeaponState)>,
) {
    for (entity, reloading, weapon_profile, mut weapon_state) in query {
        if reloading.completed_at <= time.elapsed() {
            weapon_state.ammo_current = weapon_profile.ammo_max;
            commands.entity(entity).remove::<Reloading>();
        }
    }
}

fn fire(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut WeaponInput, &WeaponProfile, &mut WeaponState)>,
) {
    for (shooter, mut weapon_input, weapon_profile, mut weapon_state) in query {
        let Some(primary_fire) = weapon_input.primary_fire else {
            continue;
        };

        let just_fired = primary_fire == InputState::JustFired;

        let should_fire = match weapon_profile.fire_mode {
            FireMode::Automatic => true,
            FireMode::SemiAutomatic => {
                weapon_input.primary_fire = Some(InputState::Fired);
                just_fired
            }
        };

        if !should_fire {
            continue;
        }

        let fire_interval = Duration::from_secs_f32(1.0 / weapon_profile.fire_rate);

        let estimated_trigger_time = if just_fired {
            time.elapsed()
        } else {
            time.elapsed() - time.delta()
        };

        let mut fire_time = Ord::max(estimated_trigger_time, weapon_state.ready_to_fire_at);

        // TODO if fire_interval <= 0 this becomes an infinite loop
        loop {
            if weapon_state.ammo_current < weapon_profile.ammo_per_shot
                || fire_time > time.elapsed()
            {
                break;
            }

            weapon_state.ready_to_fire_at = fire_time + fire_interval;
            weapon_state.ammo_current -= weapon_profile.ammo_per_shot;

            commands.trigger(ShotFired {
                shooter,
                shot_info: ShotInfo {
                    damage: weapon_profile.damage,
                    interval: fire_interval,
                    timestamp: fire_time,
                },
            });

            fire_time = weapon_state.ready_to_fire_at;
        }
    }
}

fn on_shot_fired(
    shot_fired: On<ShotFired>,
    mut commands: Commands,
    use_targeter: UseTargeter,
    spatial_query: SpatialQuery,
    team_query: Query<&Team>,
) -> Result {
    let team = team_query.get(shot_fired.shooter)?;

    let ray = use_targeter.get_ray(shot_fired.shooter, Dir3::FORWARD)?;
    let Some(hit) = spatial_query.cast_ray(
        ray.origin,
        ray.direction,
        f32::INFINITY,
        false,
        &SpatialQueryFilter::from_collision_layers(CollisionLayers::attack(team.0)),
    ) else {
        commands.trigger(ShotDone {
            shooter: shot_fired.shooter,
            shot_info: shot_fired.shot_info,
            result: ShotResult::Missed,
        });

        return Ok(());
    };

    commands.trigger(ShotHit {
        target: hit.entity,
        shot_info: shot_fired.shot_info,
        shooter: shot_fired.shooter,
    });

    Ok(())
}

fn on_shot_hit(shot_hit: On<ShotHit>, mut commands: Commands, team_query: Query<&Team>) {
    let valid_teams = match (
        team_query.get(shot_hit.target),
        team_query.get(shot_hit.shooter),
    ) {
        (Ok(target_team), Ok(shooter_team)) => target_team.0 != shooter_team.0,
        _ => false,
    };

    if !valid_teams {
        commands.trigger(ShotDone {
            shooter: shot_hit.shooter,
            shot_info: shot_hit.shot_info,
            result: ShotResult::HitObstacle,
        });

        return;
    }

    commands.trigger(ShotDone {
        shooter: shot_hit.shooter,
        shot_info: shot_hit.shot_info,
        result: ShotResult::HitTarget(shot_hit.target),
    });
}

fn ammo_on_hit(
    shot_done: On<ShotDone>,
    mut query: Query<(&WeaponProfile, &mut WeaponState)>,
) -> Result {
    let (weapon_profile, mut weapon_state) = query.get_mut(shot_done.shooter)?;

    if matches!(shot_done.result, ShotResult::HitTarget(_)) {
        weapon_state.ammo_current = u32::min(
            weapon_profile.ammo_max,
            weapon_state.ammo_current + weapon_profile.ammo_on_hit,
        );
    }

    Ok(())
}

fn ammo_on_kill(
    killed: On<Killed>,
    mut query: Query<(&WeaponProfile, &mut WeaponState)>,
) -> Result {
    let Some(killed_by) = killed.by else {
        return Ok(());
    };

    let (weapon_profile, mut weapon_state) = query.get_mut(killed_by)?;

    weapon_state.ammo_current = u32::min(
        weapon_profile.ammo_max,
        weapon_state.ammo_current + weapon_profile.ammo_on_kill,
    );

    Ok(())
}
