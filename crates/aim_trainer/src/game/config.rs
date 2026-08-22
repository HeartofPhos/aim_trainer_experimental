use bevy::prelude::*;
use derive_more::Deref;

#[derive(Resource, Deref)]
pub struct SensitivityConfig(pub schema::SensitivityConfig);
