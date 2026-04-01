use std::error::Error;
use std::fs;


use crate::ffi::*;
use crate::renderer::*;
use crate::expression::json::*;
use crate::motion::amotion::*;

pub struct ExpressionMotion {
    pub exp: Exp3,
    pub current_time: f32,
    pub weight: f32,
}

impl ExpressionMotion {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        let exp: Exp3 = serde_json::from_str(&data)?;
        Ok(Self {
            exp,
            current_time: 0.,
            weight: 0.,
        })
    }

    pub fn apply(&mut self, delta_time: f32, renderer: &mut Renderer) {
        self.current_time += delta_time;
        
        self.weight = if self.exp.fade_in_time <= 0.0 {
            1.0
        } else {
            (self.current_time / self.exp.fade_in_time).clamp(0.0, 1.0)
        };

        if self.weight <= 0.0 {
            return;
        }

        for param in &self.exp.parameters {
            unsafe {
                if let Some(idx) = renderer.find_param_index(&param.id) {
                    let param_values = csmGetParameterValues(renderer.model.model);
                    let current_val = *param_values.add(idx);

                    let new_val = match param.blend.as_str() {
                        "Multiply" => {
                            current_val * (1.0 + (param.value - 1.0) * self.weight)
                        }
                        "Overwrite" => {
                            current_val + (param.value - current_val) * self.weight
                        }
                        "Add" | _ => {
                            current_val + (param.value * self.weight)
                        }
                    };

                    *param_values.add(idx) = new_val;
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_exp3_json() {
        let exp = ExpressionMotion::new("./test_file/test.exp3.json").unwrap();
        assert_eq!(exp.exp.type_, "Live2D Expression");
        
        let p0 = Parameter {
            id: "laugh".to_string(),
            value: 1.,
            blend: "Add".to_string(),
        };
        assert_eq!(exp.exp.parameters[0], p0);
    }
}
