use bevy::prelude::*;

pub mod auto_driver;
pub mod character_controller;
pub mod input_driver;
pub mod movement;

pub fn plugin(app: &mut App) {
    app.add_plugins(auto_driver::plugin);
    app.add_plugins(character_controller::plugin);
    app.add_plugins(input_driver::plugin);
    app.add_plugins(movement::plugin);
}
