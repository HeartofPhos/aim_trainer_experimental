use aim_core::Input;
use bevy::prelude::*;

#[derive(Default)]
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
