use crate::layers::CollisionLayersExt;
use avian3d::prelude::*;
use bevy::{platform::collections::HashMap, prelude::*};
use derive_more::Deref;
use schema::Primitive;

pub fn plugin(app: &mut App) {
    app.init_resource::<PrimitiveCache>();
    app.add_observer(on_add_brush_def);
}

#[derive(Component, Default, Clone, Deref)]
#[component(immutable)]
pub struct BrushDef(pub schema::BrushDef);

#[derive(Resource)]
pub struct PrimitiveCache {
    meshes: HashMap<Primitive, PrimitiveMesh>,
    colliders: HashMap<Primitive, Collider>,
}

impl Default for PrimitiveCache {
    fn default() -> Self {
        let mut meshes: HashMap<Primitive, PrimitiveMesh> = Default::default();
        let mut colliders: HashMap<Primitive, Collider> = Default::default();

        for primitive in Primitive::iter() {
            meshes.insert(*primitive, build_mesh(primitive));
            colliders.insert(
                *primitive,
                build_collider(primitive).expect("invalid collider"),
            );
        }

        Self { meshes, colliders }
    }
}

impl PrimitiveCache {
    pub fn get_mesh(&self, primitive: &Primitive) -> Result<&PrimitiveMesh> {
        let mesh = self.meshes.get(primitive);
        Ok(mesh.ok_or("missing mesh")?)
    }

    pub fn get_collider(&self, primitive: &Primitive) -> Result<&Collider> {
        let collider = self.colliders.get(primitive);
        Ok(collider.ok_or("missing collider")?)
    }
}

#[derive(Clone)]
pub struct PrimitiveMesh {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
}

fn build_collider(primitive: &Primitive) -> Result<Collider> {
    let collider = match primitive {
        Primitive::Cuboid => Collider::cuboid(1.0, 1.0, 1.0),
        _ => {
            let mesh = build_mesh(primitive);
            Collider::convex_hull(mesh.vertices).ok_or("invalid convex hull")?
        }
    };

    Ok(collider)
}

fn build_mesh(primitive: &Primitive) -> PrimitiveMesh {
    let (mut vertices, indices) = match primitive {
        Primitive::Cuboid => {
            let vertices = vec![
                Vec3::new(0.0, 0.0, 0.0), // 0
                Vec3::new(1.0, 0.0, 0.0), // 1
                Vec3::new(1.0, 0.0, 1.0), // 2
                Vec3::new(0.0, 0.0, 1.0), // 3
                Vec3::new(0.0, 1.0, 0.0), // 4
                Vec3::new(1.0, 1.0, 0.0), // 5
                Vec3::new(1.0, 1.0, 1.0), // 6
                Vec3::new(0.0, 1.0, 1.0), // 7
            ];

            #[rustfmt::skip]
            let indices: Vec<_> = vec![
                // down
                0, 1, 2,
                2, 3, 0,
                // up
                4, 7, 6,
                6, 5, 4,
                // forward
                0, 4, 5,
                5, 1, 0,
                // back
                3, 2, 6,
                6, 7, 3,
                // left
                0, 3, 7,
                7, 4, 0,
                // right
                1, 5, 6,
                6, 2, 1,
            ];

            (vertices, indices)
        }
        Primitive::Ramp => {
            let vertices = vec![
                Vec3::new(0.0, 0.0, 0.0), // 0
                Vec3::new(1.0, 0.0, 0.0), // 1
                Vec3::new(1.0, 0.0, 1.0), // 2
                Vec3::new(0.0, 0.0, 1.0), // 3
                Vec3::new(0.0, 1.0, 0.0), // 4
                Vec3::new(1.0, 1.0, 0.0), // 5
            ];

            #[rustfmt::skip]
            let indices: Vec<_> = vec![
                // back
                5, 1, 0,
                0, 4, 5,
                // down
                0, 1, 2,
                2, 3, 0,
                // forward
                4, 3, 2,
                2, 5, 4,
                // left
                0, 3, 4,
                // right
                5, 2, 1,
            ];

            (vertices, indices)
        }
        Primitive::Corner => {
            let vertices = vec![
                Vec3::new(0.0, 0.0, 0.0), // 0
                Vec3::new(1.0, 0.0, 0.0), // 1
                Vec3::new(0.0, 0.0, 1.0), // 2
                Vec3::new(0.0, 1.0, 0.0), // 3
            ];

            #[rustfmt::skip]
            let indices: Vec<_> = vec![
                // back
                0, 3, 1,
                // left
                2, 3, 0,
                // down
                0, 1, 2,
                // forward
                2, 1, 3,
            ];

            (vertices, indices)
        }
        Primitive::CornerInverse => {
            let vertices = vec![
                Vec3::new(1.0, 0.0, 1.0), // 0
                Vec3::new(0.0, 1.0, 1.0), // 1
                Vec3::new(0.0, 0.0, 1.0), // 2
                Vec3::new(1.0, 1.0, 0.0), // 3
                Vec3::new(1.0, 0.0, 0.0), // 4
                Vec3::new(0.0, 1.0, 0.0), // 5
                Vec3::new(0.0, 0.0, 0.0), // 6
            ];

            #[rustfmt::skip]
            let indices: Vec<_> = vec![
                4, 3, 0,
                1, 6, 2,
                5, 4, 6,
                0, 6, 4,
                1, 0, 3,
                3, 5, 1,
                2, 0, 1,
                1, 5, 6,
                5, 3, 4,
                0, 2, 6,
            ];

            (vertices, indices)
        }
    };

    let offset = Vec3::NEG_ONE * 0.5;
    let mat = Mat4::from_translation(offset);

    for vertex in &mut vertices {
        *vertex = mat.transform_point3(*vertex);
    }

    let vertices: Vec<_> = indices.iter().map(|i| vertices[*i as usize]).collect();
    let normals: Vec<_> = vertices
        .chunks(3)
        .flat_map(|vertices| {
            let v0 = vertices[0];
            let v1 = vertices[1];
            let v2 = vertices[2];

            [Vec3::cross(v0 - v1, v0 - v2).normalize(); 3]
        })
        .collect();

    PrimitiveMesh { vertices, normals }
}

fn brush_primitive(brush_def: &schema::BrushDef) -> Primitive {
    match brush_def {
        schema::BrushDef::Primitive { primitive, .. } => *primitive,
        schema::BrushDef::Spawn { .. } => Primitive::Cuboid,
    }
}

fn on_add_brush_def(
    add: On<Add, BrushDef>,
    mut commands: Commands,
    primitive_cache: Res<PrimitiveCache>,
    query: Query<&BrushDef>,
) -> Result {
    let brush_def = query.get(add.entity)?;
    let collider = primitive_cache.get_collider(&brush_primitive(brush_def))?;

    commands
        .entity(add.entity)
        .insert((collider.clone(), CollisionLayers::brush(&brush_def.0)));

    Ok(())
}
