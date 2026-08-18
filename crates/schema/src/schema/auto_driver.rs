use bevy_math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDriverProfile {
    pub follow: Option<Follow>,
    pub unstick: Option<Unstick>,
    pub dodge_layers: Vec<DriverLayer<UnitVector>>,
    pub impulse_layers: Vec<DriverLayer<MagnitudeVector>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Follow {
    pub target: FollowTarget,
    pub distance: TargetDistance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FollowTarget {
    Player,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TargetDistance {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverLayer<T>(pub Vec<Variant<T>>);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde_with::serde_as]
pub enum UnitVector {
    Sphere,
    Circle(Vec3),
    #[serde(untagged)]
    Value(Vec3),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MagnitudeVector(pub UnitVector, pub f32);

#[serde_with::serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Variant<T> {
    pub weight: f32,
    pub min: f32,
    pub max: f32,
    pub value: T,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Unstick {
    pub min: f32,
    pub max: f32,
    pub speed_threshold: f32,
}
