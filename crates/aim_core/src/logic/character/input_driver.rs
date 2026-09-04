use crate::{
    Input,
    logic::{DriverSet, character::movement::MovementInput},
};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, input.in_set(DriverSet));
}

#[derive(Component)]
pub struct InputDriver;

fn input(input: Res<Input>, query: Query<&mut MovementInput, With<InputDriver>>) {
    for mut mi in query {
        mi.pitch_delta = f32::to_radians(-input.look.y);
        mi.yaw_delta = f32::to_radians(-input.look.x);

        mi.direction = Dir3::new(input.movement).ok();
    }
}
