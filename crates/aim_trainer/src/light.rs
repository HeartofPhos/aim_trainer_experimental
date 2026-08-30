use crate::shader::ShaderLoc;
use raylib::prelude::*;

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LightType {
    Directional = 0,
    Point = 1,
}

impl From<LightType> for i32 {
    fn from(value: LightType) -> Self {
        value as i32
    }
}

#[derive(Default, Clone, Copy)]
pub struct LightLoc {
    pub enabled: ShaderLoc<bool, i32>,
    pub ty: ShaderLoc<LightType, i32>,
    pub position: ShaderLoc<Vector3>,
    pub target: ShaderLoc<Vector3>,
    pub color: ShaderLoc<Vector4>,
}

pub struct LightShader<const MAX_LIGHTS: usize> {
    shader: Shader,
    pub view_pos: ShaderLoc<Vector3>,
    pub ambient: ShaderLoc<Vector4>,
    pub lights: [LightLoc; MAX_LIGHTS],
}

impl<const MAX_LIGHTS: usize> LightShader<MAX_LIGHTS> {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let shader = rl.load_shader(
            thread,
            Some("assets/shaders/lighting.vs.glsl"),
            Some("assets/shaders/lighting.fs.glsl"),
        );

        let mut lights = [LightLoc::default(); MAX_LIGHTS];

        for i in 0..MAX_LIGHTS {
            lights[i] = LightLoc {
                enabled: ShaderLoc::new(&shader, &format!("lights[{}].enabled", i)),
                ty: ShaderLoc::new(&shader, &format!("lights[{}].type", i)),
                position: ShaderLoc::new(&shader, &format!("lights[{}].position", i)),
                target: ShaderLoc::new(&shader, &format!("lights[{}].target", i)),
                color: ShaderLoc::new(&shader, &format!("lights[{}].color", i)),
            };
        }

        Self {
            view_pos: ShaderLoc::new(&shader, "viewPos"),
            ambient: ShaderLoc::new(&shader, "ambient"),
            lights,
            shader,
        }
    }

    pub fn upload(&mut self) {
        self.view_pos.upload(&mut self.shader);
        self.ambient.upload(&mut self.shader);

        for i in 0..MAX_LIGHTS {
            self.lights[i].enabled.upload(&mut self.shader);
            self.lights[i].ty.upload(&mut self.shader);
            self.lights[i].position.upload(&mut self.shader);
            self.lights[i].target.upload(&mut self.shader);
            self.lights[i].color.upload(&mut self.shader);
        }
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }
}
