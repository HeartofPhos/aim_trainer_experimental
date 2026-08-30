use raylib::prelude::*;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub struct ShaderLoc<V: Clone + Copy, S: ShaderV + From<V> = V> {
    loc: i32,
    value: Option<V>,
    _p: PhantomData<(V, S)>,
}

impl<V: Clone + Copy, S: ShaderV + From<V>> Default for ShaderLoc<V, S> {
    fn default() -> Self {
        Self {
            loc: -1,
            value: Default::default(),
            _p: Default::default(),
        }
    }
}

impl<V: Clone + Copy, S: ShaderV + From<V>> ShaderLoc<V, S> {
    pub fn new(shader: &Shader, name: &str) -> Self {
        Self {
            loc: shader.get_shader_location(name),
            value: None,
            _p: Default::default(),
        }
    }

    pub fn set(&mut self, v: V) {
        self.value = Some(v);
    }

    pub fn upload(&self, shader: &mut Shader) {
        if let Some(value) = self.value {
            shader.set_shader_value(self.loc, S::from(value));
        }
    }
}
