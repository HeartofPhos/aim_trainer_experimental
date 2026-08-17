use crate::Layer;
use glam::{Quat, Vec3};
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
    Primitive(PrimitiveDef),
    Spawn(SpawnDef),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveDef {
    pub theme: ThemeToken,
    pub primitive: Primitive,
    #[serde(default = "Default::default")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<Layer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnDef {
    pub group: SpawnGroup,
}

impl Default for BrushDef {
    fn default() -> Self {
        Self::Primitive(PrimitiveDef {
            theme: ThemeToken::Primary,
            primitive: Primitive::Cuboid,
            exclude: Default::default(),
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnGroup(pub usize);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct BrushTransform {
    pub bounds: BrushBounds,
    pub facing: Quat,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct BrushBounds {
    pub min: Vec3,
    pub max: Vec3,
}

impl BrushBounds {
    pub fn union(&self, rhs: &Self) -> Self {
        Self {
            min: Vec3::min(self.min, rhs.min),
            max: Vec3::max(self.max, rhs.max),
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn extents(&self) -> Vec3 {
        (self.max - self.min).abs()
    }
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
