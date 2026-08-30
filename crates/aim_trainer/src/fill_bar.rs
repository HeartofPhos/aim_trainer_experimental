use crate::shader::ShaderLoc;
use raylib::prelude::*;

pub struct FillBarShader {
    shader: Shader,
    pub border_radius: ShaderLoc<f32>,
    pub rounding_factor: ShaderLoc<f32>,
    pub fill_color: ShaderLoc<Vector4>,
    pub border_color: ShaderLoc<Vector4>,
    pub empty_color: ShaderLoc<Vector4>,
    pub fill: ShaderLoc<f32>,
    pub fill_axis: ShaderLoc<i32>,
    pub fill_flip: ShaderLoc<i32>,
}

impl FillBarShader {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let shader = rl.load_shader(
            thread,
            Some("assets/shaders/fill_bar.vs.glsl"),
            Some("assets/shaders/fill_bar.fs.glsl"),
        );

        Self {
            border_radius: ShaderLoc::new(&shader, "borderRadius"),
            rounding_factor: ShaderLoc::new(&shader, "roundingFactor"),
            fill_color: ShaderLoc::new(&shader, "fillColor"),
            border_color: ShaderLoc::new(&shader, "borderColor"),
            empty_color: ShaderLoc::new(&shader, "emptyColor"),
            fill: ShaderLoc::new(&shader, "fill"),
            fill_axis: ShaderLoc::new(&shader, "fillAxis"),
            fill_flip: ShaderLoc::new(&shader, "fillFlip"),
            shader,
        }
    }

    pub fn upload(&mut self) {
        self.border_radius.upload(&mut self.shader);
        self.rounding_factor.upload(&mut self.shader);
        self.fill_color.upload(&mut self.shader);
        self.border_color.upload(&mut self.shader);
        self.empty_color.upload(&mut self.shader);
        self.fill.upload(&mut self.shader);
        self.fill_axis.upload(&mut self.shader);
        self.fill_flip.upload(&mut self.shader);
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }
}
