use std::sync::Arc;
use std::collections::VecDeque;

pub enum Shader {
    Text(Arc<str>), 
    Char(Box<[char]>),
}

pub struct ShaderManager {
    shaders: VecDeque<Shader>,
    idx: usize,
}

impl ShaderManager {
    pub fn new() -> Self {
        let shaders = VecDeque::from(vec![
            Shader::Char(Box::new([' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'])),
            Shader::Text("HELLO".into()),
            Shader::Char(Box::new(['⠀', '⠁', '⠃', '⠇', '⠧', '⠷', '⠿', '⡿', '⣿'])),
            Shader::Char(Box::new([' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'])),
        ]);
        Self {
            shaders,
            idx: 0
        }
    }

    pub fn current_shader(&self) -> &Shader {
        &self.shaders[self.idx]
    }

    pub fn next(&mut self) {
        self.idx += 1;
        self.idx %= self.shaders.len();
    } 

    pub fn insert_hd(&mut self, shader: Shader) {
        self.shaders.push_front(shader);
    }
} 
