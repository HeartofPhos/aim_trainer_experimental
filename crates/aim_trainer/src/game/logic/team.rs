use bevy::prelude::*;

#[derive(Component, Default, Clone, Copy)]
#[component(immutable)]
pub struct Team(pub schema::Team);
