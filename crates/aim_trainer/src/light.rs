use raylib::prelude::*;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LightType {
    Directional = 0,
    Point = 1,
}

#[derive(Clone, Copy)]
pub struct Light {
    pub light_type: LightType,
    pub enabled: bool,
    pub position: Vector3,
    pub target: Vector3,
    pub color: Vector4,
}

#[derive(Default, Clone, Copy)]
struct LightLoc {
    enabled_loc: i32,
    type_loc: i32,
    position_loc: i32,
    target_loc: i32,
    color_loc: i32,
}

pub struct LightShader<const MAX_LIGHTS: usize> {
    shader: Shader,
    view_loc: i32,
    ambient_loc: i32,
    light_locs: [LightLoc; MAX_LIGHTS],
}

impl<const MAX_LIGHTS: usize> LightShader<MAX_LIGHTS> {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let shader = rl.load_shader(
            thread,
            Some("assets/shaders/lighting.vs.glsl"),
            Some("assets/shaders/lighting.fs.glsl"),
        );

        let ambient_loc = shader.get_shader_location("ambient");
        let view_loc = shader.get_shader_location("viewPos");

        let mut light_locs = [LightLoc::default(); MAX_LIGHTS];

        for i in 0..MAX_LIGHTS {
            light_locs[i] = LightLoc {
                enabled_loc: shader.get_shader_location(&format!("lights[{}].enabled", i)),
                type_loc: shader.get_shader_location(&format!("lights[{}].type", i)),
                position_loc: shader.get_shader_location(&format!("lights[{}].position", i)),
                target_loc: shader.get_shader_location(&format!("lights[{}].target", i)),
                color_loc: shader.get_shader_location(&format!("lights[{}].color", i)),
            };
        }

        Self {
            shader,
            view_loc,
            ambient_loc,
            light_locs,
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn set_view_pos(&mut self, p: Vector3) {
        self.shader.set_shader_value(self.view_loc, p);
    }

    pub fn set_ambient(&mut self, c: Vector4) {
        self.shader.set_shader_value(self.ambient_loc, c);
    }

    pub fn set_light(&mut self, i: usize, light: Light) {
        let light_loc = self.light_locs[i];

        self.shader
            .set_shader_value(light_loc.enabled_loc, light.enabled as i32);
        self.shader
            .set_shader_value(light_loc.type_loc, light.light_type as i32);
        self.shader
            .set_shader_value(light_loc.position_loc, light.position);
        self.shader
            .set_shader_value(light_loc.target_loc, light.target);
        self.shader
            .set_shader_value(light_loc.color_loc, light.color);
    }
}
