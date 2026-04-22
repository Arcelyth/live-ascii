pub struct ShaderManager {
    shaders: Vec<Box<[char]>>,
    idx: usize,
}

impl ShaderManager {
    pub fn new() -> Self {
        let shaders: Vec<Box<[char]>> = vec![
            Box::new([' ', '.', ':', '-', '=', '+', '*', '#', '%', '@']),
            Box::new(['⠀', '⠁', '⠃', '⠇', '⠧', '⠷', '⠿', '⡿', '⣿']),
            Box::new([' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'])
        ];
        Self {
            shaders,
            idx: 0
        }
    }

    pub fn current_shader(&self) -> &Box<[char]> {
        &self.shaders[self.idx]
    }

    pub fn next(&mut self) {
        self.idx += 1;
        self.idx %= self.shaders.len();
    } 
} 
