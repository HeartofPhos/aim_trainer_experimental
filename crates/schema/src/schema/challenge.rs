use serde::{Deserialize, Serialize};
use std::time::Duration;

#[serde_with::serde_as]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChallengeProfile {
    pub damage: Option<BufferProfile>,
    pub efficiency: Option<BufferProfile>,
    pub collect: Option<CollectProfile>,
}

#[serde_with::serde_as]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct BufferProfile {
    pub target: f32,
    #[serde_as(as = "serde_with::DurationSecondsWithFrac<f64>")]
    pub window: Duration,
}

#[serde_with::serde_as]
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct CollectProfile {
    #[serde_as(as = "serde_with::DurationSecondsWithFrac<f64>")]
    pub fill_time: Duration,
    #[serde_as(as = "serde_with::DurationSecondsWithFrac<f64>")]
    pub drain_time: Duration,
    pub speed_threshold: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChallengeScaling(pub f32);
