use bevy::prelude::*;
use rand::RngExt;

// TODO https://www.keithschwarz.com/darts-dice-coins/
pub fn weighted_random<T>(
    total_weight: f32,
    iter: impl IntoIterator<Item = (f32, T)>,
    rng: &mut impl RngExt,
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

pub fn fold<T>(opt: Option<T>, value: T, merge: impl Fn(T, T) -> T) -> T {
    opt.into_iter().fold(value, merge)
}

#[expect(unused)]
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

pub use maybe::Maybe;
mod maybe {
    use bevy::ecs::{
        component::{Immutable, StorageType},
        lifecycle::HookContext,
        prelude::*,
        system::Command,
        world::DeferredWorld,
    };
    use core::marker::PhantomData;

    // Copied from https://github.com/Leafwing-Studios/i-cant-believe-its-not-bsn/blob/46e1580857355cd3f4f997a656762244a76e80b2/src/maybe.rs

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Maybe<B: Bundle>(pub Option<B>);

    impl<B: Bundle> Component for Maybe<B> {
        type Mutability = Immutable;

        /// This is a sparse set component as it's only ever added and removed, never iterated over.
        const STORAGE_TYPE: StorageType = StorageType::SparseSet;

        fn on_add() -> Option<bevy::ecs::lifecycle::ComponentHook> {
            Some(maybe_hook::<B>)
        }
    }

    impl<B: Bundle> Maybe<B> {
        /// Creates a new `Maybe` component of type `B` with no bundle.
        pub const NONE: Self = Self(None);

        /// Creates a new `Maybe` component with the given bundle.
        #[expect(dead_code, reason = "util")]
        pub const fn new(bundle: B) -> Self {
            Self(Some(bundle))
        }

        /// Returns the contents of the `Maybe` component, if any.
        pub fn into_inner(self) -> Option<B> {
            self.0
        }
    }

    impl<B: Bundle> Default for Maybe<B> {
        /// Defaults to [`Maybe::NONE`].
        fn default() -> Self {
            Self::NONE
        }
    }

    /// A hook that runs whenever [`Maybe`] is added to an entity.
    ///
    /// Generates a [`MaybeCommand`].
    fn maybe_hook<B: Bundle>(
        mut world: DeferredWorld<'_>,
        HookContext { entity, .. }: HookContext,
    ) {
        // Component hooks can't perform structural changes, so we need to rely on commands.
        world.commands().queue(MaybeCommand {
            entity,
            _phantom: PhantomData::<B>,
        });
    }

    struct MaybeCommand<B> {
        entity: Entity,
        _phantom: PhantomData<B>,
    }

    impl<B: Bundle> Command for MaybeCommand<B> {
        type Out = ();

        fn apply(self, world: &mut World) {
            let Ok(mut entity_mut) = world.get_entity_mut(self.entity) else {
                cfg_select! {
                    debug_assertions => panic!("Entity with Maybe component not found"),
                    _ => {}
                };
            };

            let Some(maybe_component) = entity_mut.take::<Maybe<B>>() else {
                cfg_select! {
                    debug_assertions => panic!("Maybe component not found"),
                    _ => {}
                };
            };

            if let Some(bundle) = maybe_component.into_inner() {
                entity_mut.insert(bundle);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[derive(Component)]
        struct A;

        #[derive(Bundle)]
        struct TestBundle {
            maybe_a: Maybe<A>,
        }

        #[test]
        fn maybe_some() {
            let mut world = World::new();
            let entity = world
                .spawn(TestBundle {
                    maybe_a: Maybe::new(A),
                })
                .id();

            // FIXME: this should not be needed!
            world.flush();

            assert!(world.get::<A>(entity).is_some());
            assert!(world.get::<Maybe<A>>(entity).is_none());
        }

        #[test]
        fn maybe_none() {
            let mut world = World::new();
            let entity = world
                .spawn(TestBundle {
                    maybe_a: Maybe::NONE,
                })
                .id();

            // FIXME: this should not be needed!
            world.flush();

            assert!(world.get::<A>(entity).is_none());
            assert!(world.get::<Maybe<A>>(entity).is_none());
        }

        #[test]
        fn maybe_system() {
            use bevy::ecs::system::RunSystemOnce;

            let mut world = World::new();

            let entity_with_component = world
                .run_system_once(|mut commands: Commands| -> Entity {
                    commands
                        .spawn(TestBundle {
                            maybe_a: Maybe::new(A),
                        })
                        .id()
                })
                .unwrap();

            let entity_ref = world.get_entity(entity_with_component).unwrap();
            assert!(entity_ref.contains::<A>());
            assert!(!entity_ref.contains::<Maybe<A>>());

            let entity_without_component = world
                .run_system_once(|mut commands: Commands| -> Entity {
                    commands
                        .spawn(TestBundle {
                            maybe_a: Maybe::NONE,
                        })
                        .id()
                })
                .unwrap();

            let entity_ref = world.get_entity(entity_without_component).unwrap();
            assert!(!entity_ref.contains::<A>());
            assert!(!entity_ref.contains::<Maybe<A>>());
        }
    }
}

#[macro_export]
macro_rules! relationship {
    ($vis: vis, one_one $(, $args: tt)*) => {
          $crate::relationship!(
            entity,
            bevy::prelude::Entity,
            pub fn entity(&self) -> bevy::prelude::Entity {
                self.0
            },
            $vis
            $(, $args)*
        );
    };
    ($vis: vis, one_many $(, $args: tt)*) => {
        $crate::relationship!(
            entities,
            Vec<bevy::prelude::Entity>,
            pub fn entities(&self) -> &Vec<bevy::prelude::Entity> {
                &self.0
            },
            $vis
            $(, $args)*
        );
    };
    ($target_name: ident, $target_type: ty, $get_fn: item, $vis: vis, $rel: ident, [$($rel_args: meta),*], $rel_target: ident, [$($rel_target_args: meta),*]) => {
        #[derive(Debug, bevy::prelude::Component, bevy::prelude::FromTemplate)]
        #[relationship(relationship_target = $rel_target $(, $rel_args)*)]
        $vis struct $rel(pub bevy::prelude::Entity);

        #[derive(Debug, bevy::prelude::Component)]
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
