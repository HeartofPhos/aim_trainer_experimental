use aim_core::Input;
use bevy::prelude::*;

#[derive(Debug, Default, PartialEq)]
pub struct InputAggregator {
    input: Input,
    buffer_fire: bool,
}

impl InputAggregator {
    pub fn push(&mut self, input: Input) {
        self.input.look += input.look;
        self.input.movement = input.movement;
        self.input.fire = input.fire;

        if input.fire {
            self.buffer_fire = true;
        }
    }

    pub fn take(&mut self) -> Input {
        let output = Input {
            look: self.input.look,
            movement: self.input.movement,
            fire: self.input.fire || self.buffer_fire,
        };

        self.input.look = Vec2::ZERO;
        self.buffer_fire = false;

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_fire() {
        let mut input_aggregator = InputAggregator::default();

        input_aggregator.push(Input {
            fire: true,
            ..Default::default()
        });

        input_aggregator.push(Input {
            fire: false,
            ..Default::default()
        });

        assert_eq!(
            input_aggregator.take(),
            Input {
                fire: true,
                ..Default::default()
            }
        );

        assert_eq!(
            input_aggregator.take(),
            Input {
                fire: false,
                ..Default::default()
            }
        );
    }

    #[test]
    fn fire() {
        let mut input_aggregator = InputAggregator::default();

        input_aggregator.push(Input {
            fire: false,
            ..Default::default()
        });

        input_aggregator.push(Input {
            fire: true,
            ..Default::default()
        });

        assert_eq!(
            input_aggregator.take(),
            Input {
                fire: true,
                ..Default::default()
            }
        );

        assert_eq!(
            input_aggregator.take(),
            Input {
                fire: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn look() {
        let mut input_aggregator = InputAggregator::default();

        input_aggregator.push(Input {
            look: Vec2::ONE,
            ..Default::default()
        });

        input_aggregator.push(Input {
            look: Vec2::ZERO,
            ..Default::default()
        });

        assert_eq!(
            input_aggregator.take(),
            Input {
                look: Vec2::ONE,
                ..Default::default()
            }
        );

        assert_eq!(
            input_aggregator.take(),
            Input {
                look: Vec2::ZERO,
                ..Default::default()
            }
        );
    }

    #[test]
    fn movement() {
        let mut input_aggregator = InputAggregator::default();

        input_aggregator.push(Input {
            movement: Vec3::ONE,
            ..Default::default()
        });

        input_aggregator.push(Input {
            movement: Vec3::ZERO,
            ..Default::default()
        });

        assert_eq!(
            input_aggregator.take(),
            Input {
                movement: Vec3::ZERO,
                ..Default::default()
            }
        );

        assert_eq!(
            input_aggregator.take(),
            Input {
                movement: Vec3::ZERO,
                ..Default::default()
            }
        );
    }

    const LOOK: &[Vec2] = &[Vec2::ONE, Vec2::ZERO, Vec2::NEG_ONE];
    const MOVEMENT: &[Vec3] = &[Vec3::ONE, Vec3::ZERO, Vec3::NEG_ONE];
    const FIRE: &[bool] = &[true, false];

    #[test]
    fn chain() {
        let mut input_aggregator = InputAggregator::default();

        input_aggregator.push(Input {
            movement: Vec3::ONE,
            look: Vec2::ONE,
            fire: true,
        });

        input_aggregator.push(Input {
            movement: Vec3::ZERO,
            look: Vec2::ZERO,
            fire: false,
        });

        let take_expected = input_aggregator.take();
        input_aggregator.push(take_expected);
        let take_actual = input_aggregator.take();

        assert_eq!(take_expected, take_actual);

        input_aggregator.push(Input {
            movement: Vec3::ZERO,
            look: Vec2::ZERO,
            fire: false,
        });

        input_aggregator.push(Input {
            movement: Vec3::ONE,
            look: Vec2::ONE,
            fire: false,
        });

        let take_expected = input_aggregator.take();
        input_aggregator.push(take_expected);
        let take_actual = input_aggregator.take();

        assert_eq!(take_expected, take_actual);
    }
}
