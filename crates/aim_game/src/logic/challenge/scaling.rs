use crate::{
    logic::{
        SpawnSet, TimeFactor,
        challenge::{Challenge, ChallengeSet},
        movement::MovementProfile,
        shape::Shape,
    },
    utils::Maybe,
};
use bevy::prelude::*;
use core::f32::consts::PI;

pub fn plugin(app: &mut App) {
    // TODO there is a class of bug where scaling is applied 1 frame late because `apply` runs before `spawn`
    // can be solved with observers assuming challenge is always available when `spawn` is called
    app.add_systems(
        FixedUpdate,
        (
            apply::<Shape>
                .after(ChallengeSet::Update)
                .after(SpawnSet::Spawn)
                .before(SpawnSet::Move),
            apply::<TimeFactor>.after(ChallengeSet::Update),
        ),
    );

    app.add_observer(update_exponent::<Shape>);
    app.add_observer(update_exponent::<TimeFactor>);
}

fn update_exponent<T: ScalableComponent>(
    add: On<Add, Base<T>>,
    mut query: Query<(&mut ChallengeExponent, &Base<T>)>,
) -> Result {
    let (mut multiplier, base) = query.get_mut(add.entity).ignore()?;

    multiplier.0 += base.1;

    Ok(())
}

#[derive(Component, Default, Clone, Copy)]
pub struct ChallengeExponent(pub f32);

#[derive(Component, Default, Clone, Copy)]
#[component(immutable)]
#[require(ChallengeExponent)]
struct Base<T: Scalable>(pub T, f32);

pub trait Scalable: Sized {
    const EXPONENT: f32;
    fn scale(self, factor: f32) -> Self;
}

// TODO jank name
pub trait ScalableComponent: Scalable + Sized + Component + Copy {
    fn with_scaling(self, scaling: Option<schema::ChallengeScaling>) -> impl Bundle;
}

impl<T: Scalable + Sized + Component + Copy + Unpin> ScalableComponent for T {
    fn with_scaling(self, scaling: Option<schema::ChallengeScaling>) -> impl Bundle {
        let base = scaling.and_then(|x| {
            if x.0 != 0.0 {
                Some(Base(self, x.0))
            } else {
                None
            }
        });

        (self, Maybe(base))
    }
}

fn apply<T: ScalableComponent>(
    mut commands: Commands,
    challenge: Res<Challenge>,
    query: Query<(Entity, &Base<T>)>,
) {
    for (entity, base) in query {
        let factor = ops::powf(challenge.value().0, base.1 * T::EXPONENT);
        commands.entity(entity).try_insert(base.0.scale(factor));
    }
}

impl Scalable for Shape {
    const EXPONENT: f32 = -1.0;

    fn scale(self, factor: f32) -> Self {
        let mut shape: schema::Shape = self.into();

        match &mut shape {
            // a = π r^2
            // a f = π R^2 /. {R -> r x, a -> π r^2}
            schema::Shape::Sphere { radius } => {
                *radius *= ops::sqrt(factor);
            }
            // a = π r^2 + 2r (h - 2r)
            // a f = π R^2 + 2R (h - 2R) /. {R -> r x, a -> π r^2 + 2r (h - 2r)}
            schema::Shape::Capsule { radius, height } => {
                let f = factor;
                let h = *height;
                let r = *radius;

                #[rustfmt::skip]
                let factor = ((-8.0 * f * h * r + 2.0 * PI * f * h * r + 16.0 * f * r * r + PI * PI * f * r * r - 8.0 * PI * f * r * r + h * h).sqrt() - h) / ((PI - 4.0) * r);

                *radius *= factor;
            }
            // a = r h
            // a f = R h /. {R -> r x, a -> r h}
            schema::Shape::Cylinder { radius, .. } => {
                *radius *= factor;
            }
        }

        shape.into()
    }
}

impl Scalable for TimeFactor {
    const EXPONENT: f32 = 1.0;

    fn scale(self, factor: f32) -> Self {
        TimeFactor(self.0 * factor)
    }
}

impl Scalable for MovementProfile {
    const EXPONENT: f32 = 1.0;

    fn scale(self, factor: f32) -> Self {
        MovementProfile(schema::MovementProfile {
            max_speed: self.max_speed * factor,
            stop_speed: self.stop_speed * factor,
            ..self.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use bevy::math::FloatPow;

    fn calculate_area(shape: schema::Shape) -> f32 {
        match shape {
            schema::Shape::Sphere { radius } => PI * radius.squared(),
            schema::Shape::Capsule { radius, height } => {
                let circle = PI * radius.squared();
                let diameter = radius * 2.0;
                let rectangle = diameter * (height - diameter);

                circle + rectangle
            }
            schema::Shape::Cylinder { radius, height } => radius * 2.0 * height,
        }
    }

    #[test]
    fn shape_scaling() {
        let shapes = [
            Shape::Sphere(Sphere::new(0.5)),
            Shape::Sphere(Sphere::new(1.0)),
            Shape::Capsule(Capsule3d::new(0.5, 2.0)),
            Shape::Capsule(Capsule3d::new(1.0, 2.0)),
            Shape::Cylinder(Cylinder::new(0.5, 1.0)),
            Shape::Cylinder(Cylinder::new(1.0, 1.5)),
        ];

        let factors = [
            ops::powf(1.1, -3.0),
            ops::powf(1.1, -2.0),
            ops::powf(1.1, -1.0),
            ops::powf(1.1, 0.0),
            ops::powf(1.1, 1.0),
            ops::powf(1.1, 2.0),
            ops::powf(1.1, 3.0),
        ];

        for shape in shapes {
            let base_area = calculate_area(shape.into());

            for factor in factors {
                let expected_area = base_area * factor;

                let scaled_shape = shape.scale(factor);
                let actual_area = calculate_area(scaled_shape.into());

                assert_abs_diff_eq!(expected_area, actual_area, epsilon = 0.00001);
            }
        }
    }

    #[test]
    fn capsule_max_factor() {
        // sphere
        let shape = Shape::Capsule(Capsule3d::new(0.5, 0.0));
        let scaled_shape = shape.scale(1000.0);

        assert_eq!(shape, scaled_shape);
    }
}
