use glam::{Mat4, Vec3};
use raylib::prelude::*;
use schema::Primitive;

use crate::light::{Light, LightShader, LightType};

mod light;

fn main() {
    let (scenario, _): (schema::Scenario, _) =
        ref_asset::io::read_file(ref_asset::paths::scenario("track-move")).unwrap();

    let (mut rl, thread) = raylib::init().fullscreen().title("aim_trainer").build();

    let meshes: Vec<_> = scenario
        .level
        .brush_list
        .iter()
        .filter_map(|b| match &b.def {
            schema::BrushDef::Primitive {
                theme,
                primitive,
                exclude,
            } => Some((build_mesh(&thread, primitive), b.transform)),
            _ => None,
        })
        .collect();

    let mut light_shader = LightShader::<1>::new(&mut rl, &thread);
    let brightness = Vector4::new(0.1, 0.1, 0.1, 1.0);
    light_shader.set_ambient(Vector4::from(Color::WHITE) * brightness);
    light_shader.set_light(
        0,
        Light {
            light_type: LightType::Directional,
            enabled: true,
            position: Vector3::new(2.0, 3.0, 2.0),
            target: Vector3::zero(),
            color: Vector4::from(Color::WHITE) * brightness,
        },
    );

    let mut mat = rl.load_material_default(&thread);
    mat.set_map_color(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO, Color::WHITE);
    mat.set_shader(light_shader.shader());

    let mut camera = Camera3D::perspective(
        Vector3::new(0.0, 2.0, 4.0), // Camera position
        Vector3::new(0.0, 2.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        60.0,                        // Camera field-of-view Y
    );

    let camera_mode = CameraMode::CAMERA_FIRST_PERSON;

    rl.disable_cursor();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        camera.update_camera(camera_mode);

        light_shader.set_view_pos(camera.position);

        d.clear_background(Color::GRAY);

        {
            let mut c = d.begin_mode3D(camera);

            for (mesh, transform) in &meshes {
                let matrix = Matrix::compose(
                    vec_to_vector(transform.bounds.center()),
                    Quaternion::identity(),
                    vec_to_vector(transform.bounds.extents()),
                );

                c.draw_mesh(mesh, mat.clone(), matrix);
            }
        }

        d.draw_fps(10, 10);
    }
}

fn vec_to_vector(v: Vec3) -> Vector3 {
    Vector3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

fn build_mesh(thread: &RaylibThread, primitive: &Primitive) -> Mesh {
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
        // Primitive::Corner => {
        //     let vertices = vec![
        //         Vec3::new(0.0, 0.0, 0.0), // 0
        //         Vec3::new(1.0, 0.0, 0.0), // 1
        //         Vec3::new(0.0, 0.0, 1.0), // 2
        //         Vec3::new(0.0, 1.0, 0.0), // 3
        //     ];

        //     #[rustfmt::skip]
        //         let indices: Vec<_> = vec![
        //             // back
        //             0, 3, 1,
        //             // left
        //             2, 3, 0,
        //             // down
        //             0, 1, 2,
        //             // forward
        //             2, 1, 3,
        //         ];

        //     (vertices, indices)
        // }
        // Primitive::CornerInverse => {
        //     let vertices = vec![
        //         Vec3::new(1.0, 0.0, 1.0),
        //         Vec3::new(0.0, 1.0, 1.0),
        //         Vec3::new(0.0, 0.0, 1.0),
        //         Vec3::new(1.0, 1.0, 0.0),
        //         Vec3::new(1.0, 0.0, 0.0),
        //         Vec3::new(0.0, 1.0, 0.0),
        //         Vec3::new(0.0, 0.0, 0.0),
        //     ];

        //     #[rustfmt::skip]
        //         let indices: Vec<_> = vec![
        //             4, 3, 0,
        //             1, 6, 2,
        //             5, 4, 6,
        //             0, 6, 4,
        //             1, 0, 3,
        //             3, 5, 1,
        //             2, 0, 1,
        //             1, 5, 6,
        //             5, 3, 4,
        //             0, 2, 6,
        //         ];

        //     (vertices, indices)
        // }
        _ => (Vec::new(), Vec::new()),
    };

    let offset = Vec3::NEG_ONE * 0.5;
    let mat = Mat4::from_translation(offset);

    for vertex in &mut vertices {
        *vertex = mat.transform_point3(*vertex);
    }

    let normals: Vec<_> = indices
        .chunks(3)
        .flat_map(|i| {
            let v0 = vertices[i[0] as usize];
            let v1 = vertices[i[1] as usize];
            let v2 = vertices[i[2] as usize];

            [Vec3::cross(v0 - v1, v0 - v2).normalize(); 3]
        })
        .collect();

    let vertices: Vec<_> = indices.iter().map(|i| vertices[*i as usize]).collect();

    let vertices: Vec<_> = vertices.into_iter().map(vec_to_vector).collect();
    let normals: Vec<_> = normals.into_iter().map(vec_to_vector).collect();
    let tex: Vec<Vector2> = vertices.iter().map(|_| Vector2::zero()).collect();

    match Mesh::gen_mesh(&vertices, &tex)
        .normals(&normals)
        .build(thread)
    {
        Ok(mesh) => mesh,
        Err(err) => {
            println!("{:#?}", err);
            panic!()
        }
    }
}
