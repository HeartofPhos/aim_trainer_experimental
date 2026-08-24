use crate::{
    game::{Game, Input, plugins::level::PrimitiveCache},
    light::{Light, LightShader, LightType},
};
use bevy::math::prelude::*;
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

    let time_step = Duration::from_secs_f32(1.0 / 256.0);
    let game = Game::new(ref_asset::paths::scenario("track-move"), time_step);

    let (mut rl, thread) = raylib::init().title("aim_trainer").log_to_rust().build();
    rl.toggle_borderless_windowed();

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

    let draw_brush = build_draw_brush(&thread, game.primitive_cache(), mat_a, mat_b);

    let mut camera = Camera3D::perspective(
        Vector3::ZERO, // Camera position
        Vector3::ZERO, // Camera looking at point
        Vector3::Y,    // Camera up vector (rotation towards target)
        60.0,          // Camera field-of-view Y
    );

    rl.disable_cursor();

    game_loop(
        rl,
        time_step,
        game,
        Input::default(),
        |rl, game, input, elapsed, delta_time| {
            game.update(delta_time, *input);
            input.look = Default::default();
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
    primitive_cache: &PrimitiveCache,
    mat_a: WeakMaterial,
    mat_b: WeakMaterial,
) -> impl Fn(&mut RaylibMode3D<RaylibDrawHandle>, &BrushDef, &BrushTransform) + use<> {
    let primitive_cache = raylib_primitive_cache(thread, primitive_cache);

    move |c, brush_def, brush_transform| {
        let (mat, mesh) = match brush_def {
            schema::BrushDef::Primitive { primitive, .. } => {
                (&mat_a, primitive_cache.get(primitive))
            }
            schema::BrushDef::Spawn { .. } => (&mat_b, primitive_cache.get(&Primitive::Cuboid)),
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

fn raylib_primitive_cache(
    thread: &RaylibThread,
    primitive_cache: &PrimitiveCache,
) -> HashMap<Primitive, Mesh> {
    let mut lookup = HashMap::new();
    for primitive in Primitive::iter() {
        let mesh = build_mesh(thread, primitive, primitive_cache);
        lookup.insert(*primitive, mesh);
    }

    lookup
}

fn build_mesh(
    thread: &RaylibThread,
    primitive: &Primitive,
    primitive_cache: &PrimitiveCache,
) -> Mesh {
    let mesh = primitive_cache.get_mesh(primitive).expect("missing mesh");

    let vertices: Vec<_> = mesh.vertices.iter().copied().map(Into::into).collect();
    let normals: Vec<_> = mesh.normals.iter().copied().map(Into::into).collect();
    let tex: Vec<Vector2> = vertices.iter().map(|_| Vector2::zero()).collect();

    Mesh::gen_mesh(&vertices, &tex)
        .normals(&normals)
        .build(thread)
        .expect("invalid mesh")
}
