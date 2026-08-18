use serde::{Deserialize, Serialize};
use std::time::Duration;

mod auto_driver;
mod challenge;
mod character;
mod layer;
mod level;
mod team;

pub use auto_driver::*;
pub use challenge::*;
pub use character::*;
pub use layer::*;
pub use level::*;
pub use team::*;

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub level: LevelData,
    pub player: PlayerTemplate,
    pub weapon: WeaponProfile,
    #[serde_as(as = "serde_with::DurationSecondsWithFrac<f64>")]
    pub timeout_delay: Duration,
    pub challenge: ChallengeProfile,
    pub bot_template: Option<BotTemplate>,
    pub collectable_template: Option<CollectableTemplate>,
}

#[derive(Serialize, Deserialize)]
pub struct SensitivityConfig {
    pub sensitivity: f32,
    pub sensitivity_factor: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Real {
    Infinity,
    #[serde(untagged)]
    Value(f32),
}

impl From<Real> for f32 {
    fn from(value: Real) -> Self {
        match value {
            Real::Infinity => f32::INFINITY,
            Real::Value(value) => value,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct SpawnRules {
    pub limit: usize,
}

#[serde_with::serde_as]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct WeaponProfile {
    pub damage: f32,
    // TODO validate fire_interval > 0 to avoid infinite shots per frame
    pub fire_rate: f32,
    pub fire_mode: FireMode,
    pub ammo_max: u32,
    pub ammo_per_shot: u32,
    pub ammo_on_hit: u32,
    pub ammo_on_kill: u32,
    #[serde_as(as = "serde_with::DurationSecondsWithFrac<f64>")]
    pub reload_time: Duration,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum FireMode {
    #[default]
    Automatic,
    SemiAutomatic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerTemplate {
    pub character: CharacterTemplate,
    pub movement: MovementProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectableTemplate {
    pub spawn_rules: SpawnRules,
    pub shape: Shape,
    pub character: CharacterTemplate,
    pub movement: MovementProfile,
    pub driver: AutoDriverProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotTemplate {
    pub spawn_rules: SpawnRules,
    pub health: Real,
    pub character: CharacterTemplate,
    pub movement: MovementProfile,
    pub size_scaling: ChallengeScaling,
    pub time_scaling: ChallengeScaling,
    pub driver: AutoDriverProfile,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct MovementProfile {
    pub max_speed: f32,
    pub stop_speed: f32,
    pub gravity: f32,
    pub accelerate: f32,
    pub air_accelerate: f32,
    pub friction: f32,
    pub air_friction: f32,
    pub mode: MovementMode,
    pub race: bool,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum MovementMode {
    #[default]
    Fly,
    Jump {
        speed: f32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Shape {
    Sphere { radius: f32 },
    Capsule { radius: f32, height: f32 },
    Cylinder { radius: f32, height: f32 },
}
