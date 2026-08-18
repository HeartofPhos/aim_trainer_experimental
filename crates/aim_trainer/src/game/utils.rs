use bevy_math::prelude::*;
use rand::{Rng, RngExt};

// TODO https://www.keithschwarz.com/darts-dice-coins/
pub fn weighted_random<T>(
    total_weight: f32,
    iter: impl IntoIterator<Item = (f32, T)>,
    rng: &mut impl Rng,
) -> Option<T> {
    let mut random_weight = rng.random::<f32>() * total_weight;

    for (weight, value) in iter {
        if random_weight < weight {
            return Some(value);
        }

        random_weight -= weight;
    }

    None
}

pub trait Direction {
    const LEFT: Self;
    const RIGHT: Self;
    const UP: Self;
    const DOWN: Self;
    const FORWARD: Self;
    const BACK: Self;
}

impl Direction for Vec3 {
    const LEFT: Self = Self::NEG_X;
    const RIGHT: Self = Self::X;
    const UP: Self = Self::Y;
    const DOWN: Self = Self::NEG_Y;
    const FORWARD: Self = Self::NEG_Z;
    const BACK: Self = Self::Z;
}

impl Direction for IVec3 {
    const LEFT: Self = Self::NEG_X;
    const RIGHT: Self = Self::X;
    const UP: Self = Self::Y;
    const DOWN: Self = Self::NEG_Y;
    const FORWARD: Self = Self::NEG_Z;
    const BACK: Self = Self::Z;
}

impl Direction for Dir3 {
    const LEFT: Self = Self::NEG_X;
    const RIGHT: Self = Self::X;
    const UP: Self = Self::Y;
    const DOWN: Self = Self::NEG_Y;
    const FORWARD: Self = Self::NEG_Z;
    const BACK: Self = Self::Z;
}

#[macro_export]
macro_rules! relationship {
    ($vis: vis, one_one $(, $args: tt)*) => {
          $crate::relationship!(
            entity,
            bevy_ecs::prelude::Entity,
            pub fn entity(&self) -> bevy_ecs::prelude::Entity {
                self.0
            },
            $vis
            $(, $args)*
        );
    };
    ($vis: vis, one_many $(, $args: tt)*) => {
        $crate::relationship!(
            entities,
            Vec<bevy_ecs::prelude::Entity>,
            pub fn entities(&self) -> &Vec<bevy_ecs::prelude::Entity> {
                &self.0
            },
            $vis
            $(, $args)*
        );
    };
    ($target_name: ident, $target_type: ty, $get_fn: item, $vis: vis, $rel: ident, [$($rel_args: meta),*], $rel_target: ident, [$($rel_target_args: meta),*]) => {
        #[derive(Debug, bevy_ecs::prelude::Component, bevy_ecs::prelude::FromTemplate)]
        #[relationship(relationship_target = $rel_target $(, $rel_args)*)]
        $vis struct $rel(pub bevy_ecs::prelude::Entity);

        #[derive(Debug, bevy_ecs::prelude::Component)]
        #[relationship_target(relationship = $rel $(, $rel_target_args)*)]
        $vis struct $rel_target($target_type);

        impl $rel_target {
            #[allow(clippy::allow_attributes, dead_code, reason = "macro generated")]
            $get_fn
        }
    };
}

#[macro_export]
macro_rules! relationships {
    (pub $($inner:tt)*) => {
        pub use relationships::*;
        mod relationships {
           $($inner)*
        }
    };
    ($($inner:tt)*) => {
        use relationships::*;
        mod relationships {
           $($inner)*
        }
    };
}
