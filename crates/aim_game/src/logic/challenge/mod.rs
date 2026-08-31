use crate::{
    logic::{
        challenge::buffer::{BufferProfile, BufferTransform, UseBuffer},
        weapon::{ShotDone, ShotInfo, ShotResult},
    },
    utils::Maybe,
};
use bevy::prelude::*;

mod buffer;
mod collect;
mod core;
mod scaling;

pub use collect::*;
pub use core::*;
pub use scaling::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(core::plugin);
    app.add_plugins(collect::plugin);
    app.add_plugins(scaling::plugin);
    app.add_plugins(buffer::plugin::<DamageProfile>);
    app.add_plugins(buffer::plugin::<EfficiencyProfile>);

    app.add_observer(on_shot_done);
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChallengeSet {
    Accumulate,
    Update,
}

#[derive(Component, Deref, Default, Clone, Copy)]
pub struct DamageProfile(pub schema::BufferProfile);

impl BufferProfile for DamageProfile {
    const TRANSFORM: BufferTransform = BufferTransform::NOOP;
}
#[derive(Component, Deref, Default, Clone, Copy)]
pub struct EfficiencyProfile(pub schema::BufferProfile);

fn prob_to_odds(mut prob: f32) -> f32 {
    prob = f32::clamp(prob, 0.0, 1.0);
    prob / (1.0 - prob)
}

fn odds_to_prob(odds: f32) -> f32 {
    odds / (1.0 + odds)
}

impl BufferProfile for EfficiencyProfile {
    const TRANSFORM: BufferTransform = BufferTransform::new(prob_to_odds, odds_to_prob);
}

pub fn challenge_bundle(challenge_profile: schema::ChallengeProfile) -> impl Bundle {
    let damage = challenge_profile.damage.map(DamageProfile);
    let efficiency = challenge_profile.efficiency.map(EfficiencyProfile);
    let collect = challenge_profile.collect.map(CollectProfile);

    (Maybe(damage), Maybe(efficiency), Maybe(collect))
}

fn on_shot_done(
    shot_done: On<ShotDone>,
    challenge: Res<Challenge>,
    challenge_exponent_query: Query<&ChallengeExponent>,
    mut query: Query<AnyOf<(UseBuffer<DamageProfile>, UseBuffer<EfficiencyProfile>)>>,
) -> Result {
    let (damage_buffer, efficiency_buffer) = query.get_mut(shot_done.shooter).ignore()?;

    let ShotResult::HitTarget(target) = shot_done.result else {
        return Ok(());
    };

    let ShotInfo {
        damage,
        interval,
        timestamp,
    } = shot_done.shot_info;

    let challenge_exponent = challenge_exponent_query
        .get(target)
        .copied()
        .unwrap_or_default();
    let challenge = ChallengeValue(ops::powf(challenge.value().0, challenge_exponent.0));

    if let Some(mut damage_buffer) = damage_buffer {
        damage_buffer.push(damage, challenge, timestamp)?;
    }

    if let Some(mut efficiency_buffer) = efficiency_buffer {
        efficiency_buffer.push(interval.as_secs_f32(), challenge, timestamp)?;
    }

    Ok(())
}
