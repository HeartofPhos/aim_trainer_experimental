use bevy::prelude::*;
use derive_more::Deref;

#[derive(Component, Default, Clone, Deref)]
#[component(immutable)]
pub struct BrushDef(pub schema::BrushDef);

#[derive(Component, Default, Clone, Copy, Deref)]
#[component(immutable)]
pub struct BrushTransform(pub schema::BrushTransform);
