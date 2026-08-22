use crate::{
    game::{Game, Input},
    light::{Light, LightShader, LightType},
};
use bevy::prelude::*;
use raylib::prelude::*;
use schema::{BrushDef, BrushTransform, Primitive};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

mod game;
mod light;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let game = Game::new(ref_asset::paths::scenario("track-move"));

    let (mut rl, thread) = raylib::init()
        .fullscreen()
        .title("aim_trainer")
        .log_to_rust()
        .build();

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

    let mut mat_a = rl.load_material_default(&thread);
    mat_a.set_map_color(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO, Color::WHITE);
    mat_a.set_shader(light_shader.shader());

    let mut mat_b = rl.load_material_default(&thread);
    mat_b.set_map_color(
        ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO,
        Color::GREEN.alpha(0.1),
    );
    mat_b.set_shader(light_shader.shader());

    let draw_brush = build_draw_brush(&thread, mat_a, mat_b);

    let mut camera = Camera3D::perspective(
        Vector3::ZERO, // Camera position
        Vector3::ZERO, // Camera looking at point
        Vector3::Y,    // Camera up vector (rotation towards target)
        60.0,          // Camera field-of-view Y
    );

    rl.disable_cursor();

    game_loop(
        rl,
        Duration::from_secs_f32(1.0 / 256.0),
        game,
        Input::default(),
        |rl, game, input, elapsed, delta_time| {
            game.update(delta_time, *input);
            *input = Input::default();
        },
        |rl, game, input, elapsed, delta_time| {
            fn dir(neg: bool, pos: bool) -> f32 {
                match (neg, pos) {
                    (true, false) => -1.0,
                    (false, true) => 1.0,
                    _ => 0.0,
                }
            }
            input.look += Vec2::from(rl.get_mouse_delta());
            input.movement = Vec3 {
                x: dir(
                    rl.is_key_down(KeyboardKey::KEY_A),
                    rl.is_key_down(KeyboardKey::KEY_D),
                ),
                y: dir(
                    rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL),
                    rl.is_key_down(KeyboardKey::KEY_SPACE),
                ),
                z: dir(
                    rl.is_key_down(KeyboardKey::KEY_W),
                    rl.is_key_down(KeyboardKey::KEY_S),
                ),
            }
            .normalize_or(Vec3::ZERO);

            let mut d = rl.begin_drawing(&thread);

            let Ok(player) = game.player() else {
                return;
            };

            camera.target = (player.translation + player.rotation * Vec3::NEG_Z).into();
            camera.position = player.translation.into();
            light_shader.set_view_pos(camera.position);

            d.clear_background(Color::GRAY);

            {
                let mut c = d.begin_mode3D(camera);

                game.level_brushes(|brush_def, brush_transform| {
                    draw_brush(&mut c, brush_def, brush_transform)
                });
            }

            d.draw_fps(10, 10);
        },
    );
}

fn game_loop<S, I>(
    mut rl: RaylibHandle,
    timestep: Duration,
    mut state: S,
    mut input_state: I,
    mut integrate: impl FnMut(&mut RaylibHandle, &mut S, &mut I, Duration, Duration),
    mut render: impl FnMut(&mut RaylibHandle, &S, &mut I, Duration, Duration),
) {
    let mut elapsed = Duration::ZERO;
    let mut accumulator = Duration::ZERO;

    let mut current_time = Instant::now();

    while !rl.window_should_close() {
        let new_time = Instant::now();
        let delta_time = new_time.duration_since(current_time);
        current_time = new_time;

        accumulator += delta_time;

        while accumulator > timestep {
            integrate(&mut rl, &mut state, &mut input_state, elapsed, timestep);
            accumulator -= timestep;
            elapsed += timestep;
        }

        render(&mut rl, &state, &mut input_state, elapsed, delta_time);
    }
}

fn build_draw_brush(
    thread: &RaylibThread,
    mat_a: WeakMaterial,
    mat_b: WeakMaterial,
) -> impl Fn(&mut RaylibMode3D<RaylibDrawHandle>, &BrushDef, &BrushTransform) {
    let primitive_lookup = primitive_lookup(thread);

    move |c, brush_def, brush_transform| {
        let (mat, mesh) = match brush_def {
            schema::BrushDef::Primitive { primitive, .. } => {
                (&mat_a, primitive_lookup.get(primitive))
            }
            schema::BrushDef::Spawn { .. } => (&mat_b, primitive_lookup.get(&Primitive::Cuboid)),
        };

        if let Some(mesh) = mesh {
            c.draw_mesh(
                mesh,
                mat.clone(),
                Matrix::compose(
                    brush_transform.translation.into(),
                    brush_transform.rotation.into(),
                    brush_transform.scale.into(),
                ),
            );
        }
    }
}

fn primitive_lookup(thread: &RaylibThread) -> HashMap<Primitive, Mesh> {
    let mut lookup = HashMap::new();
    for primitive in Primitive::iter() {
        let mesh = build_mesh(thread, primitive);
        lookup.insert(*primitive, mesh);
    }

    lookup
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
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(0.0, 1.0, 1.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
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

    let vertices: Vec<_> = vertices.into_iter().map(Into::into).collect();
    let normals: Vec<_> = normals.into_iter().map(Into::into).collect();
    let tex: Vec<Vector2> = vertices.iter().map(|_| Vector2::zero()).collect();

    Mesh::gen_mesh(&vertices, &tex)
        .normals(&normals)
        .build(thread)
        .expect("invalid mesh")
}
