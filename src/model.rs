use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Model3 {
    pub version: usize,
    #[serde(default)]
    pub file_references: FileRef,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub hit_areas: Vec<HitArea>,
    #[serde(default)]
    pub layout: Option<Layout>,
}

impl Model3 {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct FileRef {
    pub moc: Option<PathBuf>,
    #[serde(default)]
    pub textures: Vec<PathBuf>,
    #[serde(default)]
    pub physics: Option<PathBuf>,
    #[serde(default)]
    pub display_info: Option<PathBuf>,
    #[serde(default)]
    pub pose: Option<PathBuf>,
    #[serde(default)]
    pub expressions: Vec<Expression>,
    #[serde(default)]
    pub motions: HashMap<String, Vec<MotionRef>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Group {
    name: String,
    ids: Vec<String>,
    target: Target,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialOrd, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum Target {
    Parameter,
    Part,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialOrd, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Expression {
    pub name: String,
    pub file: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct MotionRef {
    pub file: String,
    pub sound: Option<String>,
    #[serde(default = "crate::utils::default_fade_time")]
    pub fade_in_time: f32,
    #[serde(default = "crate::utils::default_fade_time")]
    pub fade_out_time: f32,
}



#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct HitArea {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialOrd, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Layout {
    center_x: f32,
    center_y: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_model3_json() {
        let model3 = Model3::new("./test_file/test.model3.json").unwrap();
        let res = Model3 {
            version: 3,
            file_references: FileRef {
                moc: Some("model.moc3".into()),
                textures: vec!["texture_00.png".into()],
                physics: Some("model.physics3.json".into()),
                display_info: None, 
                pose: Some("model.pose3.json".into()),
                expressions: vec![
                    Expression {
                        name: "smile".into(),
                        file: "exp_smile.json".into(),
                    },
                    Expression {
                        name: "angry".into(),
                        file: "exp_angry.json".into(),
                    },
                ],
                motions: {
                    let mut map = HashMap::new();
                    map.insert(
                        "Idle".into(),
                        vec![MotionRef {
                            file: "idle.motion3.json".into(),
                            sound: None,
                            fade_in_time: 1.0,
                            fade_out_time: 1.0,
                        }],
                    );
                    map.insert(
                        "TapBody".into(),
                        vec![MotionRef {
                            file: "tap_body.motion3.json".into(),
                            sound: None,
                            fade_in_time: 1.0,
                            fade_out_time: 1.0,
                        }],
                    );
                    map
                },
            },
            groups: vec![
                Group {
                    target: Target::Parameter,
                    name: "EyeBlink".into(),
                    ids: vec!["ParamEyeLOpen".into(), "ParamEyeROpen".into()],
                },
                Group {
                    target: Target::Parameter,
                    name: "LipSync".into(),
                    ids: vec!["ParamMouthOpenY".into()],
                },
            ],
            hit_areas: vec![
                HitArea {
                    id: "HitAreaHead".into(),
                    name: "head".into(),
                },
                HitArea {
                    id: "HitAreaBody".into(),
                    name: "body".into(),
                },
            ],
            layout: None
        };
        assert_eq!(model3, res);
    }
}
