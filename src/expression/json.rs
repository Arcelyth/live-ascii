use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Exp3 {
    pub type_: String,
    #[serde(default = "crate::utils::default_fade_time")]
    pub fade_in_time: f32,
    #[serde(default = "crate::utils::default_fade_time")]
    pub fade_out_time: f32,
    pub parameters: Vec<Parameter>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Parameter {
    pub id: String,
    pub value: f32,
    #[serde(default = "default_blend_type")]
    pub blend: String,
}

fn default_blend_type() -> String {
    "Add".to_string()
}
