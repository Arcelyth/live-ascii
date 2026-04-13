use std::error::Error;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsJson {
    pub version: usize,
    pub meta: PhysicsMeta,
    #[serde(rename = "PhysicsSettings")]
    pub settings: Vec<PhysicsSetting>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsMeta {
    #[serde(rename = "PhysicsSettingCount")]
    pub setting_count: usize,
    #[serde(rename = "TotalInputCount")]
    pub input_count: usize,
    #[serde(rename = "TotalOutputCount")]
    pub output_count: usize,
    pub vertex_count: usize,
    pub fps: usize,
    pub effective_forces: Force,
    #[serde(rename = "PhysicsDictionary")]
    pub dict: Vec<DictEntry>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Force {
    gravity: Vector2,
    wind: Vector2,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct DictEntry {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Source {
    pub target: String,
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct NormalizationValue {
    pub minimum: f32,
    pub maximum: f32,
    pub default: f32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Normalization {
    pub position: NormalizationValue,
    pub angle: NormalizationValue,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsInput {
    pub source: Source,
    #[serde(rename = "Type")]
    pub kind: String,
    pub weight: f32,
    pub reflect: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsOutput {
    pub destination: Source,
    pub vertex_index: i32,
    pub scale: f32,
    pub weight: f32,
    #[serde(rename = "Type")]
    pub kind: String,
    pub reflect: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Vector2 {
    x: f32,
    y: f32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsVertex {
    pub position: Vector2,
    pub mobility: f32,
    pub delay: f32,
    pub acceleration: f32,
    pub radius: f32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct PhysicsSetting {
    pub id: String,
    pub input: Vec<PhysicsInput>,
    pub output: Vec<PhysicsOutput>,
    pub vertices: Vec<PhysicsVertex>,
    pub normalization: Normalization,
}

impl PhysicsJson {
    pub fn from_path(base_dir: &str, path: &str) -> Result<Self, Box<dyn Error>> {
        let full_path = Path::new(base_dir).join(path);
        let data = fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read file {:?}: {}", full_path, e))?;

        let p: PhysicsJson = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse JSON ({:?}): {}", full_path, e))?;
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_physic3_json() {
        let expected = PhysicsJson {
            version: 3,
            meta: PhysicsMeta {
                setting_count: 16,
                input_count: 43,
                output_count: 20,
                vertex_count: 33,
                fps: 30,
                effective_forces: Force {
                    gravity: Vector2 { x: 0.0, y: -1.0 },
                    wind: Vector2 { x: 0.0, y: 0.0 },
                },
                dict: vec![
                    DictEntry {
                        id: "PhysicsSetting1".to_string(),
                        name: "Hair Sway_Front".to_string(),
                    },
                    DictEntry {
                        id: "PhysicsSetting2".to_string(),
                        name: "Hair Sway_Side".to_string(),
                    },
                ],
            },
            settings: vec![PhysicsSetting {
                id: "PhysicsSetting1".to_string(),
                input: vec![PhysicsInput {
                    source: Source {
                        target: "Parameter".to_string(),
                        id: "ParamAngleX".to_string(),
                    },
                    kind: "X".to_string(),
                    weight: 60.0,
                    reflect: false,
                }],
                output: vec![PhysicsOutput {
                    destination: Source {
                        target: "Parameter".to_string(),
                        id: "ParamHairFront".to_string(),
                    },
                    vertex_index: 1,
                    scale: 1.0,
                    weight: 100.0,
                    kind: "Angle".to_string(),
                    reflect: false,
                }],
                vertices: vec![PhysicsVertex {
                    position: Vector2 { x: 0.0, y: 10.0 },
                    mobility: 0.95,
                    delay: 0.8,
                    acceleration: 1.12,
                    radius: 10.0,
                }],
                normalization: Normalization {
                    position: NormalizationValue {
                        minimum: -10.0,
                        default: 0.0,
                        maximum: 10.0,
                    },
                    angle: NormalizationValue {
                        minimum: -10.0,
                        default: 0.0,
                        maximum: 10.0,
                    },
                },
            }],
        };

        let result = PhysicsJson::from_path("./test_file/", "test.physics3.json")
            .expect("Failed to parse the json file");

        assert_eq!(result, expected);
    }
}
