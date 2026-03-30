use std::error::Error;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::ffi::*;
use crate::renderer::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Exp3 {
    pub type_: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Parameter {
    pub id: String,
    pub value: f32,
    pub blend: String,
}

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && (self.value == other.value) && (self.blend == other.blend)
    }
}

pub struct Expression {
    pub exp: Exp3,
    pub weight: f32,
    pub speed: f32,
}

impl Expression {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        let exp = serde_json::from_str(&data)?;
        Ok(Self {
            exp,
            weight: 0.,
            speed: 1.,
        })
    }

    //    pub fn apply(&mut self, delta_time: f32, renderer: &mut Renderer) {
    //        if self.weight < 1. {
    //            self.weight = (self.weight + delta_time * self.speed).min(1.);
    //        }
    //        for param in &self.exp.parameters {
    //            unsafe {
    //                if let Some(idx) = renderer.find_param_index(&param.id) {
    //                    let param_values = csmGetParameterValues(renderer.model);
    //                    let current_val = *param_values.add(idx);
    //
    //                    let new_val = match param.blend.as_str() {
    //                        "Add" => current_val + param.value,
    //                        "Multiply" => current_val * param.value,
    //                        "Overwrite" => param.value,
    //                        _ => param.value,
    //                    };
    //
    //                    *param_values.add(idx) = current_val + (new_val - current_val) * self.weight;
    //                }
    //            }
    //        }
    //    }
    pub fn apply(&mut self, delta_time: f32, renderer: &mut Renderer) {
        if self.weight < 1.0 {
            self.weight = (self.weight + delta_time * self.speed).min(1.0);
        }

        unsafe {
            let p_count = csmGetParameterCount(renderer.model) as usize;
            let p_vs = csmGetParameterValues(renderer.model);
            let p_default_vs = csmGetParameterDefaultValues(renderer.model);

            for i in 0..p_count {
                let id = renderer.get_param_id_by_index(i);

                let target_param = self.exp.parameters.iter().find(|p| p.id == id);

                let current_val = *p_vs.add(i);

                if let Some(param) = target_param {
                    let new_val = match param.blend.as_str() {
                        "Add" => current_val + param.value,
                        "Multiply" => current_val * param.value,
                        "Overwrite" => param.value,
                        _ => param.value,
                    };
                    *p_vs.add(i) = current_val + (new_val - current_val) * self.weight;
                } else {
                    let default_val = *p_default_vs.add(i);
                    if (current_val - default_val).abs() > 0.001 {
                        *p_vs.add(i) = current_val + (default_val - current_val) * self.weight;
                    }
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
        let exp = Expression::new("./test_file/test.exp3.json").unwrap();
        assert_eq!(exp.exp.type_, "Live2D Expression");
        let p0 = Parameter {
            id: "laugh".to_string(),
            value: 1.,
            blend: "Add".to_string(),
        };
        assert_eq!(exp.exp.parameters[0], p0);
    }
}
