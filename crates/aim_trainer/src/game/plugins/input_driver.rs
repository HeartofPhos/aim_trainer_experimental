use crate::game::{
    Input, Update,
    config::SensitivityConfig,
    plugins::{InputSet, movement::MovementInput},
};
use bevy_ecs::prelude::*;
use bevy_math::prelude::*;

pub fn plugin(world: &mut World) {
    world
        .get_resource_or_init::<Schedules>()
        .add_systems(Update, input.in_set(InputSet));
}

#[derive(Component)]
pub struct InputDriver;

fn input(
    input: Res<Input>,
    sensitivty_config: Res<SensitivityConfig>,
    query: Query<&mut MovementInput, With<InputDriver>>,
) {
    for mut mi in query {
        mi.pitch_delta = f32::to_radians(-input.look.y)
            * sensitivty_config.sensitivity
            * sensitivty_config.sensitivity_factor;
        mi.yaw_delta = f32::to_radians(-input.look.x)
            * sensitivty_config.sensitivity
            * sensitivty_config.sensitivity_factor;

        mi.direction = Dir3::new(input.movement).ok();
    }
}
