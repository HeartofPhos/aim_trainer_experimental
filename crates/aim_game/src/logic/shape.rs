use avian3d::prelude::*;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_observer(on_shape_insert);
}

fn on_shape_insert(
    insert: On<Insert, Shape>,
    mut commands: Commands,
    query: Query<&Shape>,
) -> Result {
    let shape = query.get(insert.entity)?;

    let mut ec = commands.entity(insert.entity);
    let extents = shape.extents();

    ec.insert((ShapeExtents(extents), Collider::from(shape)));

    Ok(())
}

// TODO consider removing
#[derive(Component, Deref)]
#[component(immutable)]
pub struct ShapeExtents(pub Vec3);

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[component(immutable)]
pub enum Shape {
    Sphere(Sphere),
    Capsule(Capsule3d),
    Cylinder(Cylinder),
}

impl Shape {
    pub fn extents(&self) -> Vec3 {
        match *self {
            Shape::Sphere(shape) => Vec3::new(shape.radius, shape.radius, shape.radius),
            Shape::Capsule(shape) => {
                Vec3::new(shape.radius, shape.half_length + shape.radius, shape.radius)
            }
            Shape::Cylinder(shape) => Vec3::new(shape.radius, shape.half_height, shape.radius),
        }
    }

    pub fn radius(&self) -> f32 {
        match *self {
            Shape::Sphere(shape) => shape.radius,
            Shape::Capsule(shape) => shape.radius,
            Shape::Cylinder(shape) => shape.radius,
        }
    }
}

impl From<schema::Shape> for Shape {
    fn from(value: schema::Shape) -> Self {
        match value {
            schema::Shape::Sphere { radius } => Shape::Sphere(Sphere::new(radius)),
            schema::Shape::Capsule { radius, height } => {
                let half_height = height * 0.5;
                let radius = f32::min(half_height, radius);
                Shape::Capsule(Capsule3d {
                    radius,
                    half_length: half_height - radius,
                })
            }
            schema::Shape::Cylinder { radius, height } => Shape::Cylinder(Cylinder {
                radius,
                half_height: height * 0.5,
            }),
        }
    }
}

impl From<&Shape> for Collider {
    fn from(value: &Shape) -> Self {
        match value {
            Shape::Sphere(shape) => Collider::sphere(shape.radius),
            Shape::Capsule(shape) => Collider::capsule(shape.radius, shape.half_length * 2.0),
            Shape::Cylinder(shape) => Collider::cylinder(shape.radius, shape.half_height * 2.0),
        }
    }
}
