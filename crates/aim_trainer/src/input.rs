use bevy_math::prelude::*;

#[derive(Default)]
pub struct InputAggregator {
    input: aim_game::Input,
    buffer_fire: bool,
}

impl InputAggregator {
    pub fn push(&mut self, input: aim_game::Input) {
        self.input.look += input.look;
        self.input.movement = input.movement;
        self.input.fire = input.fire;

        if input.fire {
            self.buffer_fire = true;
        }
    }

    pub fn take(&mut self) -> aim_game::Input {
        let output = aim_game::Input {
            look: self.input.look,
            movement: self.input.movement,
            fire: self.input.fire || self.buffer_fire,
        };

        self.input.look = Vec2::ZERO;
        self.buffer_fire = false;

        output
    }
}
