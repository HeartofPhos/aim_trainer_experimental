use crate::{
    fill_bar::FillBarShader,
    light::{LightShader, LightType},
};
use aim_bridge::prelude::*;
use bevy_math::prelude::*;
use raylib::prelude::*;
use schema::{BrushDef, BrushTransform, Primitive};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

mod fill_bar;
mod light;
mod shader;

struct SensitivityConfig {
    sensitivity: f32,
    sensitivity_factor: f32,
}

const DEFAULT_FILL_BAR_SIZE: Vec3 = Vec3::new(0.75, 0.15, 1.0);
const FILL_BAR_OFFSET: Vec3 = Vec3::new(0.0, 0.05, 0.0);

fn main() {
    let time_step = Duration::from_secs_f32(1.0 / 256.0);
    let scenario_path = ref_asset::paths::scenario("track-move");
    let (scenario, _): (schema::Scenario, _) =
        ref_asset::io::read_file(scenario_path).expect("failed to load scenario");

    let mut bridge = AimBridge::new(AimCore {
        scenario,
        challenge: Default::default(),
        timestep: time_step,
        seed: [0; 8],
    });
    let sensitivity_config = SensitivityConfig {
        sensitivity: 1.0,
        sensitivity_factor: 0.022,
    };

    let (mut rl, thread) = raylib::init().title("aim_trainer").log_to_rust().build();
    rl.toggle_borderless_windowed();

    let mut light_shader = LightShader::<1>::new(&mut rl, &thread);
    {
        let brightness = Vector4::new(0.1, 0.1, 0.1, 1.0);

        light_shader
            .ambient
            .set(Vector4::from(Color::WHITE) * brightness);

        let light = &mut light_shader.lights[0];
        light.ty.set(LightType::Directional);
        light.enabled.set(true);
        light.position.set(Vector3::new(2.0, 3.0, 2.0));
        light.target.set(Vector3::zero());
        light.color.set(Vector4::from(Color::WHITE) * brightness);
    }

    let mut mat_a = rl.load_material_default(&thread);
    mat_a.set_map_color(ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO, Color::WHITE);
    mat_a.set_shader(light_shader.shader());

    let mut mat_b = rl.load_material_default(&thread);
    mat_b.set_map_color(
        ffi::MaterialMapIndex::MATERIAL_MAP_ALBEDO,
        Color::GREEN.alpha(0.1),
    );
    mat_b.set_shader(light_shader.shader());

    let mut fill_bar_shader = FillBarShader::new(&mut rl, &thread);
    {
        fill_bar_shader.border_radius.set(0.01);
        fill_bar_shader.rounding_factor.set(1.0);
        fill_bar_shader
            .fill_color
            .set(Vector4::new(1.0, 1.0, 1.0, 1.0));
        fill_bar_shader
            .border_color
            .set(Vector4::new(0.1, 0.1, 0.1, 1.0));
        fill_bar_shader
            .empty_color
            .set(Vector4::new(0.1, 0.1, 0.1, 1.0));
        fill_bar_shader.fill.set(0.1);
        fill_bar_shader.fill_axis.set(0);
        fill_bar_shader.fill_flip.set(0);

        fill_bar_shader.upload();
    }

    let mut fill_bar_mat = rl.load_material_default(&thread);
    fill_bar_mat.set_shader(fill_bar_shader.shader());

    let fill_bar_mesh = build_quad(&thread);

    let draw_brush = build_draw_brush(&thread, bridge.primitive_cache(), mat_a, mat_b);

    let mut camera = Camera3D::perspective(
        Vector3::ZERO, // Camera position
        Vector3::ZERO, // Camera looking at point
        Vector3::Y,    // Camera up vector (rotation towards target)
        70.5,          // Camera field-of-view Y
    );

    rl.disable_cursor();

    while !rl.window_should_close() {
        fn dir(neg: bool, pos: bool) -> f32 {
            match (neg, pos) {
                (true, false) => -1.0,
                (false, true) => 1.0,
                _ => 0.0,
            }
        }

        let input = Input {
            look: Vec2::from(rl.get_mouse_delta())
                * sensitivity_config.sensitivity
                * sensitivity_config.sensitivity_factor,
            movement: Vec3 {
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
            .normalize_or(Vec3::ZERO),
            fire: rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT),
        };

        bridge.update(Duration::from_secs_f32(rl.get_frame_time()), input);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::GRAY);

        let Ok(camera_transform) = bridge.camera() else {
            continue;
        };

        camera.target =
            (camera_transform.translation + camera_transform.rotation * Vec3::NEG_Z).into();
        camera.position = camera_transform.translation.into();

        light_shader.view_pos.set(camera.position);
        light_shader.upload();

        {
            let mut c = d.begin_mode3D(camera);

            bridge.level_brushes(|brush_def, brush_transform| {
                if matches!(
                    brush_def,
                    BrushDef::Primitive {
                        theme: schema::ThemeToken::Invisible,
                        ..
                    } | BrushDef::Spawn { .. }
                ) {
                    return;
                }

                draw_brush(&mut c, brush_def, brush_transform)
            });

            let color = Color::new(24, 24, 24, 255);
            let resolution = 16;
            bridge.shapes(|transform, shape, render, health| {
                match (shape, render) {
                    (Shape::Sphere(shape), Render::Shape) => c.draw_sphere_ex(
                        transform.translation,
                        shape.radius,
                        resolution,
                        resolution,
                        color,
                    ),
                    (Shape::Capsule(shape), Render::Shape) => c.draw_capsule(
                        transform.translation - Vec3::new(0.0, shape.half_length, 0.0),
                        transform.translation + Vec3::new(0.0, shape.half_length, 0.0),
                        shape.radius,
                        resolution,
                        resolution,
                        color,
                    ),
                    (Shape::Cylinder(shape), Render::Shape) => c.draw_cylinder(
                        transform.translation - shape.half_height,
                        shape.radius,
                        shape.radius,
                        shape.half_height * 2.0,
                        resolution,
                        color,
                    ),
                    (_, Render::Radius) => c.draw_circle3D(
                        transform.translation + Vec3::new(0.0, -shape.extents().y, 0.0),
                        shape.radius(),
                        Vector3::X,
                        90.0,
                        Color::WHITE,
                    ),
                }

                if let Some(health) = health {
                    let scale = DEFAULT_FILL_BAR_SIZE;
                    let offset =
                        Vec3::new(0.0, shape.extents().y + scale.y * 0.5, 0.0) + FILL_BAR_OFFSET;

                    fill_bar_shader.fill.set(health.ratio());
                    fill_bar_shader.upload();

                    c.draw_mesh(
                        &fill_bar_mesh,
                        fill_bar_mat.clone(),
                        Mat4::from_scale_rotation_translation(
                            scale,
                            Quat::IDENTITY,
                            transform.translation + offset,
                        ),
                    );
                }
            });
        }

        d.draw_circle(
            d.get_render_width() / 2,
            d.get_render_height() / 2,
            4.0,
            Color::new(0, 255, 255, 255),
        );

        d.draw_fps(10, 10);
        let challenge_text = format!("{:.2}", bridge.challenge());
        let font_size = 20;
        let m = d.measure_text(&challenge_text, font_size);
        d.draw_text(
            &challenge_text,
            (d.get_screen_width() - m) / 2,
            10,
            font_size,
            Color::WHITE,
        );
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

fn build_quad(thread: &RaylibThread) -> Mesh {
    let offset = Vector3::new(-0.5, -0.5, 0.0);
    let vertices = [
        Vector3::new(1.0, 1.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(1.0, 1.0, 0.0),
    ]
    .map(|v| v + offset);

    let uv = [
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(0.0, 0.0),
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(1.0, 1.0),
    ];

    Mesh::gen_mesh(&vertices, &uv)
        .build(thread)
        .expect("invalid mesh")
}
