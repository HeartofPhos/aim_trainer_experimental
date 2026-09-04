use avian3d::prelude::*;
use schema::{BrushDef, Layer, Team};

#[extend::ext]
pub impl CollisionLayers {
    fn brush<'a>(brush_def: &BrushDef) -> Self {
        let (membership, exclude_layers) = match brush_def {
            BrushDef::Primitive { exclude, .. } => (Layer::World, Some(exclude)),
            BrushDef::Spawn { .. } => (Layer::Spawn, None),
        };

        let membership = LayerMask::from_layer(membership);
        let exclude = exclude_layers
            .map(LayerMask::from_layers)
            .unwrap_or(LayerMask::NONE);
        let filter = LayerMask::ALL ^ exclude;

        CollisionLayers::new(membership, filter)
    }

    fn character(team: Team) -> Self {
        let enemy_team = team.complement();

        CollisionLayers::new(
            LayerMask::from_layer(Layer::Character(team)),
            LayerMask::from_layers(&[Layer::World, Layer::Attack(enemy_team), Layer::Collectable]),
        )
    }

    fn attack(team: Team) -> Self {
        let enemy_team = team.complement();

        CollisionLayers::new(
            LayerMask::from_layer(Layer::Attack(team)),
            LayerMask::from_layers(&[Layer::Character(enemy_team), Layer::World]),
        )
    }

    fn collectable() -> Self {
        CollisionLayers::new(
            LayerMask::from_layer(Layer::Collectable),
            LayerMask::from_layer(Layer::World),
        )
    }

    fn collectable_sensor() -> Self {
        CollisionLayers::new(
            LayerMask::from_layer(Layer::Collectable),
            LayerMask::from_layer(Layer::Character(Team::ALL)),
        )
    }

    fn select() -> Self {
        CollisionLayers::new(
            LayerMask::ALL,
            [LayerMask::from_layers(&[Layer::World, Layer::Spawn])],
        )
    }
}

#[extend::ext]
impl LayerMask {
    fn from_layer(value: Layer) -> Self {
        Self(value.get_bits())
    }

    fn from_layers<'a>(iter: impl IntoIterator<Item = &'a Layer>) -> Self {
        let mut bits = 0;
        for value in iter.into_iter() {
            bits |= value.get_bits();
        }

        Self(bits)
    }
}
