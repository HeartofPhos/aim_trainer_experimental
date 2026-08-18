use crate::Layer;
use bevy_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LevelData {
    pub brush_list: Vec<Brush>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Brush {
    pub def: BrushDef,
    pub transform: BrushTransform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrushDef {
    Primitive {
        theme: ThemeToken,
        primitive: Primitive,
        #[serde(default = "Default::default")]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        exclude: Vec<Layer>,
    },
    Spawn {
        group: SpawnGroup,
    },
}

impl Default for BrushDef {
    fn default() -> Self {
        Self::Primitive {
            theme: ThemeToken::Primary,
            primitive: Primitive::Cuboid,
            exclude: Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnGroup(pub usize);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct BrushTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThemeToken {
    Primary,
    Secondary,
    Tertiary,
    Invisible,
}

impl ThemeToken {
    pub fn iter() -> std::slice::Iter<'static, Self> {
        const VALUES: &[ThemeToken] = &[
            ThemeToken::Primary,
            ThemeToken::Secondary,
            ThemeToken::Tertiary,
            ThemeToken::Invisible,
        ];

        VALUES.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Primitive {
    Cuboid,
    Ramp,
    Corner,
    CornerInverse,
}

impl Primitive {
    pub fn iter() -> std::slice::Iter<'static, Primitive> {
        const VALUES: &[Primitive] = &[
            Primitive::Cuboid,
            Primitive::Ramp,
            Primitive::Corner,
            Primitive::CornerInverse,
        ];

        VALUES.iter()
    }
}
