use crate::{logic::challenge::ChallengeSet, utils::fold};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, update_challenge.in_set(ChallengeSet::Update));
}

#[derive(derive_more::Debug, Clone, Copy, PartialEq, derive_more::Mul, derive_more::Div)]
#[mul(forward)]
#[div(forward)]
pub struct ChallengeValue(pub f32);

impl ChallengeValue {
    const CHALLENGE_FACTOR: Self = Self(1.1);

    #[cfg(test)]
    pub fn pow(exp: f32) -> Self {
        Self(ops::powf(Self::CHALLENGE_FACTOR.0, exp))
    }

    pub fn log(&self) -> f32 {
        let log_base_constant = 1.0 / ops::log2(Self::CHALLENGE_FACTOR.0);
        ops::log2(self.0) * log_base_constant
    }
}

impl Default for ChallengeValue {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Eq for ChallengeValue {}

impl PartialOrd for ChallengeValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for ChallengeValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        f32::total_cmp(&self.0, &other.0)
    }
}

#[derive(Resource, Clone)]
pub struct Challenge {
    value: ChallengeValue,
}

impl Challenge {
    pub fn new(value: ChallengeValue) -> Self {
        Self { value }
    }

    pub fn resume(value: ChallengeValue) -> Self {
        Self {
            value: value / ChallengeValue::CHALLENGE_FACTOR,
        }
    }

    pub fn value(&self) -> ChallengeValue {
        self.value
    }
}

#[derive(Component)]
pub struct ChallengeSource(pub Option<ChallengeValue>);

fn update_challenge(mut challenge: ResMut<Challenge>, query: Query<&mut ChallengeSource>) {
    let mut value = None;

    for mut challenge_source in query {
        if let Some(challenge_value) = challenge_source.0.take() {
            value.replace(fold(value, challenge_value.0, f32::min));
        }
    }

    if let Some(value) = value {
        challenge.value = ChallengeValue(value);
    }
}
