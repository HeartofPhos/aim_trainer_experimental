use crate::Shape;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CharacterTemplate {
    pub shape: Shape,
    pub eyes: Option<CharacterEyes>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CharacterEyes {
    pub height: f32,
    pub offset: f32,
}
