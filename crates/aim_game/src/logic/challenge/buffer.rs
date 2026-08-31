use crate::logic::challenge::{
    ChallengeSet,
    core::{Challenge, ChallengeSource, ChallengeValue},
};
use bevy::{ecs::query::QueryData, prelude::*};
use jiff::SignedDuration;
use std::{
    cmp::{max_by, min_by},
    error::Error,
    marker::PhantomData,
    ops::Deref,
    time::Duration,
};
use thiserror::Error;

pub fn plugin<P: BufferProfile>(app: &mut App) {
    app.register_required_components::<P, Buffer<P>>();
    app.register_required_components::<P, BufferMultiplier>();
    app.add_systems(
        FixedUpdate,
        (warmstart::<P>, write::<P>)
            .chain()
            .in_set(ChallengeSet::Accumulate),
    );

    app.add_observer(on_insert_buffer::<P>);
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct UseBuffer<P: BufferProfile> {
    buffer_profile: &'static P,
    buffer_multiplier: &'static BufferMultiplier,
    buffer: &'static mut Buffer<P>,
}

#[derive(Component)]
pub struct BufferMultiplier(pub f32);

impl Default for BufferMultiplier {
    fn default() -> Self {
        Self(1.0)
    }
}

impl<'w, 's, P: BufferProfile> UseBufferItem<'w, 's, P> {
    pub fn push(
        &mut self,
        value: f32,
        challenge: ChallengeValue,
        timestamp: impl TryInto<SignedDuration, Error: Error + Send + Sync + 'static>,
    ) -> Result {
        let timestamp = timestamp.try_into()?;
        let smudge = Duration::from_secs_f32(value / self.buffer_profile.target);

        let rate = self.buffer_profile.target;

        let rate = P::TRANSFORM.map(rate);
        let rate = rate * challenge.0 * self.buffer_multiplier.0;
        let rate = P::TRANSFORM.inverse().map(rate);

        self.buffer.entries.push(BufferEntry {
            rate,
            interval: Interval::new(timestamp, timestamp + smudge.try_into()?)?,
        });

        Ok(())
    }

    fn warmstart(
        &mut self,
        challenge: ChallengeValue,
        timestamp: impl TryInto<SignedDuration, Error: Error + Send + Sync + 'static>,
    ) -> Result {
        let timestamp = timestamp.try_into()?;
        let window: SignedDuration = self.buffer_profile.window.try_into()?;

        let rate = self.buffer_profile.target;

        let rate = P::TRANSFORM.map(rate);
        let rate = rate * challenge.0;
        let rate = P::TRANSFORM.inverse().map(rate);

        self.buffer.entries.push(BufferEntry {
            rate,
            interval: Interval::new(timestamp - window, timestamp)?,
        });

        Ok(())
    }

    fn calculate_challenge(
        &mut self,
        timestamp: impl TryInto<SignedDuration, Error: Error + Send + Sync + 'static>,
    ) -> Result<ChallengeValue> {
        let timestamp = timestamp.try_into()?;

        let window = self.buffer_profile.window.try_into()?;
        let interval = Interval::new(timestamp - window, timestamp)?;

        let mut net = 0.0;

        let buffer = &mut self.buffer.entries;
        for i in (0..buffer.len()).rev() {
            let entry = &buffer[i];

            if entry.interval.upper < interval.lower {
                buffer.swap_remove(i);
                continue;
            }

            if let Ok(overlap) = Interval::overlap(&entry.interval, &interval) {
                net += entry.rate * overlap.length().as_secs_f32();
            }
        }

        let actual = net / window.as_secs_f32();

        let actual = P::TRANSFORM.map(actual);
        let target = P::TRANSFORM.map(self.buffer_profile.target);

        let ratio = actual / target;

        Ok(ChallengeValue(ratio))
    }
}

#[derive(Debug, Clone, Copy)]
struct BufferEntry {
    rate: f32,
    interval: Interval,
}

#[derive(Component)]
struct Buffer<P: BufferProfile> {
    entries: Vec<BufferEntry>,
    _p: PhantomData<P>,
}

#[derive(Component, Default)]
struct Warmstart;

impl<P: BufferProfile> Default for Buffer<P> {
    fn default() -> Self {
        Self {
            entries: default(),
            _p: PhantomData,
        }
    }
}
pub trait BufferProfile: Component + Deref<Target = schema::BufferProfile> {
    const TRANSFORM: BufferTransform;
}

pub struct BufferTransform(fn(f32) -> f32, fn(f32) -> f32);

impl BufferTransform {
    pub const NOOP: Self = Self(|value| value, |value| value);

    pub const fn new(a: fn(f32) -> f32, b: fn(f32) -> f32) -> Self {
        Self(a, b)
    }

    pub const fn inverse(self) -> Self {
        Self(self.1, self.0)
    }

    pub fn map(&self, v: f32) -> f32 {
        (self.0)(v)
    }
}

#[derive(Debug, Clone, Copy)]
struct Interval {
    lower: SignedDuration,
    upper: SignedDuration,
}

#[derive(Debug, Error)]
#[error("The resulting interval would be invalid")]
struct InvalidIntervalError;

impl Interval {
    fn new(lower: SignedDuration, upper: SignedDuration) -> Result<Self, InvalidIntervalError> {
        if lower >= upper {
            return Err(InvalidIntervalError);
        };

        Ok(Interval { lower, upper })
    }

    fn overlap(&self, b: &Self) -> Result<Self, InvalidIntervalError> {
        let lower = max_by(self.lower, b.lower, Ord::cmp);
        let upper = min_by(self.upper, b.upper, Ord::cmp);

        Self::new(lower, upper)
    }

    fn length(&self) -> SignedDuration {
        self.upper - self.lower
    }
}

fn on_insert_buffer<P: BufferProfile>(add: On<Add, Buffer<P>>, mut commands: Commands) {
    commands.entity(add.entity).insert(Warmstart);
}

fn warmstart<P: BufferProfile>(
    time: Res<Time>,
    challenge: Res<Challenge>,
    query: Query<(Entity, UseBuffer<P>), With<Warmstart>>,
    mut commands: Commands,
) -> Result {
    for (entity, mut use_buffer) in query {
        use_buffer.warmstart(challenge.value(), time.elapsed())?;
        commands.entity(entity).remove::<Warmstart>();
    }

    Ok(())
}

fn write<P: BufferProfile>(
    time: Res<Time>,
    query: Query<(Entity, UseBuffer<P>)>,
    mut commands: Commands,
) -> Result {
    for (entity, mut use_buffer) in query {
        let challenge = use_buffer.calculate_challenge(time.elapsed())?;
        commands
            .entity(entity)
            .insert(ChallengeSource(Some(challenge)));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::challenge::{DamageProfile, EfficiencyProfile};
    use approx::assert_abs_diff_eq;

    fn warmstart<P: BufferProfile>(profile: P, log_challenge: f32) {
        let mut app = App::new();
        let world = app.world_mut();

        world.spawn((profile, Buffer::<P>::default(), BufferMultiplier(1.0)));

        let mut query = world.query::<UseBuffer<P>>();
        let mut use_buffer = query.single_mut(world).unwrap();

        let timestamp = SignedDuration::ZERO;
        let expected = ChallengeValue::pow(log_challenge);
        use_buffer.warmstart(expected, timestamp).unwrap();
        let actual = use_buffer.calculate_challenge(timestamp).unwrap();

        assert_abs_diff_eq!(expected.0, actual.0, epsilon = EPSILON)
    }

    fn push<P: BufferProfile>(profile: P, log_challenge: f32, segments: u32) {
        let mut app = App::new();
        let world = app.world_mut();

        world.spawn((profile, Buffer::<P>::default(), BufferMultiplier(1.0)));

        let mut query = world.query::<UseBuffer<P>>();
        let mut use_buffer = query.single_mut(world).unwrap();

        let timestamp = use_buffer.buffer_profile.window / segments;

        let expected = ChallengeValue::pow(log_challenge);

        for i in 0..segments {
            use_buffer
                .push(
                    use_buffer.buffer_profile.target * timestamp.as_secs_f32(),
                    expected,
                    timestamp * i,
                )
                .unwrap();
        }

        let actual = use_buffer
            .calculate_challenge(use_buffer.buffer_profile.window)
            .unwrap();

        assert_abs_diff_eq!(expected.0, actual.0, epsilon = EPSILON)
    }

    fn reference<P: BufferProfile>(
        profile: P,
        log_challenge: f32,
        log_spread: f32,
        multiplier: f32,
    ) {
        let mut app = App::new();
        let world = app.world_mut();

        world.spawn((
            profile,
            Buffer::<P>::default(),
            BufferMultiplier(multiplier),
        ));

        let mut query = world.query::<UseBuffer<P>>();
        let mut use_buffer = query.single_mut(world).unwrap();

        let interval = use_buffer.buffer_profile.window.as_secs_f32();

        let a = ChallengeValue::pow(log_challenge + log_spread);
        let b = ChallengeValue::pow(log_challenge - log_spread);

        // reference implementation
        let expected = {
            let net = {
                let target = P::TRANSFORM.map(use_buffer.buffer_profile.target);

                let a = target * a.0 * multiplier;
                let b = target * b.0 * multiplier;

                let a = P::TRANSFORM.inverse().map(a);
                let b = P::TRANSFORM.inverse().map(b);

                a + b
            };

            let actual = net * 0.5;

            let actual = P::TRANSFORM.map(actual);
            let target = P::TRANSFORM.map(use_buffer.buffer_profile.target);

            let ratio = actual / target;

            ChallengeValue(ratio)
        };

        let actual = {
            let value = use_buffer.buffer_profile.target * interval * 0.5;

            use_buffer.push(value, a, SignedDuration::ZERO).unwrap();
            use_buffer.push(value, b, SignedDuration::ZERO).unwrap();

            use_buffer
                .calculate_challenge(use_buffer.buffer_profile.window)
                .unwrap()
        };

        assert_abs_diff_eq!(expected.0, actual.0, epsilon = EPSILON)
    }

    const EPSILON: f32 = 0.0001;

    const CHALLENGES: &[f32] = &[
        -32.0, -16.0, -8.0, -4.0, -2.0, -1.0, 0.0, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0,
    ];

    const SEGMENTS: &[u32] = &[1, 2, 4, 8, 32];

    const MULTIPLIERS: &[f32] = &[0.5, 1.0, 2.0];

    #[test]
    fn damage() {
        let schema_buffer_profile = schema::BufferProfile {
            target: 50.0,
            window: Duration::from_secs(5),
        };

        let profile = DamageProfile(schema_buffer_profile);

        for challenge in CHALLENGES {
            warmstart(profile, *challenge);
        }

        for challenge in CHALLENGES {
            for segment in SEGMENTS {
                push(profile, *challenge, *segment);
            }
        }

        for challenge_a in CHALLENGES {
            for challenge_b in CHALLENGES {
                for multiplier in MULTIPLIERS {
                    reference(profile, *challenge_a, *challenge_b, *multiplier);
                }
            }
        }
    }

    #[test]
    fn efficiency() {
        let schema_buffer_profile = schema::BufferProfile {
            target: 0.1,
            window: Duration::from_secs(5),
        };

        let profile = EfficiencyProfile(schema_buffer_profile);

        for challenge in CHALLENGES {
            warmstart(profile, *challenge);
        }

        for challenge in CHALLENGES {
            for segment in SEGMENTS {
                push(profile, *challenge, *segment);
            }
        }

        for challenge_a in CHALLENGES {
            for challenge_b in CHALLENGES {
                for multiplier in MULTIPLIERS {
                    reference(profile, *challenge_a, *challenge_b, *multiplier);
                }
            }
        }
    }
}
