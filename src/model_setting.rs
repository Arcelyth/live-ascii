use std::collections::HashMap;
use std::path::Path;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ModelSetting {
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


impl ModelSetting {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data)?)
    }

    pub fn is_exist_model_file(&self) -> bool {
        self.file_references.moc.is_some()
    }

    pub fn is_exist_texture_files(&self) -> bool {
        !self.file_references.textures.is_empty()
    }

    pub fn is_exist_hit_areas(&self) -> bool {
        !self.hit_areas.is_empty()
    }

    pub fn is_exist_physics_file(&self) -> bool {
        self.file_references.physics.is_some()
    }

    pub fn is_exist_pose_file(&self) -> bool {
        self.file_references.pose.is_some()
    }

    pub fn is_exist_display_info_file(&self) -> bool {
        self.file_references.display_info.is_some()
    }

    pub fn is_exist_expression_file(&self) -> bool {
        !self.file_references.expressions.is_empty()
    }

    pub fn is_exist_motion_groups(&self) -> bool {
        !self.file_references.motions.is_empty()
    }

    pub fn is_exist_motion_group_name(&self, group_name: &str) -> bool {
        self.file_references.motions.contains_key(group_name)
    }

    pub fn is_exist_motion_sound_file(&self, group_name: &str, index: usize) -> bool {
        self.file_references.motions.get(group_name)
            .and_then(|group| group.get(index))
            .and_then(|motion| motion.sound.as_ref())
            .is_some()
    }

    pub fn is_exist_motion_fade_in(&self, group_name: &str, index: usize) -> bool {
        self.get_motion(group_name, index).is_some()
    }

    pub fn is_exist_motion_fade_out(&self, group_name: &str, index: usize) -> bool {
        self.get_motion(group_name, index).is_some()
    }

    fn find_group(&self, name: &str) -> Option<&Group> {
        self.groups.iter().find(|g| g.name == name)
    }

    pub fn is_exist_eye_blink_parameters(&self) -> bool {
        self.find_group("EyeBlink").is_some()
    }

    pub fn is_exist_lip_sync_parameters(&self) -> bool {
        self.find_group("LipSync").is_some()
    }

    pub fn get_model_file_name(&self) -> Option<&PathBuf> {
        self.file_references.moc.as_ref()
    }

    pub fn get_texture_count(&self) -> usize {
        self.file_references.textures.len()
    }

    pub fn get_texture_directory(&self) -> Option<&Path> {
        self.file_references.textures.first()
            .and_then(|p| p.parent())
    }

    pub fn get_texture_file_name(&self, index: usize) -> Option<&PathBuf> {
        self.file_references.textures.get(index)
    }

    pub fn get_hit_areas_count(&self) -> usize {
        self.hit_areas.len()
    }

    pub fn get_hit_area_id(&self, index: usize) -> Option<&str> {
        self.hit_areas.get(index).map(|h| h.id.as_str())
    }

    pub fn get_hit_area_name(&self, index: usize) -> Option<&str> {
        self.hit_areas.get(index).map(|h| h.name.as_str())
    }

    pub fn get_physics_file_name(&self) -> Option<&PathBuf> {
        self.file_references.physics.as_ref()
    }

    pub fn get_pose_file_name(&self) -> Option<&PathBuf> {
        self.file_references.pose.as_ref()
    }

    pub fn get_display_info_file_name(&self) -> Option<&PathBuf> {
        self.file_references.display_info.as_ref()
    }

    pub fn get_expression_count(&self) -> usize {
        self.file_references.expressions.len()
    }

    pub fn get_expression_name(&self, index: usize) -> Option<&str> {
        self.file_references.expressions.get(index).map(|e| e.name.as_str())
    }

    pub fn get_expression_file_name(&self, index: usize) -> Option<&str> {
        self.file_references.expressions.get(index).map(|e| e.file.as_str())
    }

    pub fn get_motion_group_count(&self) -> usize {
        self.file_references.motions.len()
    }

    pub fn get_motion_group_names(&self) -> Vec<&String> {
        self.file_references.motions.keys().collect()
    }

    pub fn get_motion_count(&self, group_name: &str) -> usize {
        self.file_references.motions.get(group_name).map_or(0, |m| m.len())
    }

    fn get_motion(&self, group_name: &str, index: usize) -> Option<&MotionRef> {
        self.file_references.motions.get(group_name).and_then(|group| group.get(index))
    }

    pub fn get_motion_file_name(&self, group_name: &str, index: usize) -> Option<&str> {
        self.get_motion(group_name, index).map(|m| m.file.as_str())
    }

    pub fn get_motion_sound_file_name(&self, group_name: &str, index: usize) -> Option<&str> {
        self.get_motion(group_name, index).and_then(|m| m.sound.as_deref())
    }

    pub fn get_motion_fade_in_time_value(&self, group_name: &str, index: usize) -> f32 {
        self.get_motion(group_name, index).map_or(-1.0, |m| m.fade_in_time)
    }

    pub fn get_motion_fade_out_time_value(&self, group_name: &str, index: usize) -> f32 {
        self.get_motion(group_name, index).map_or(-1.0, |m| m.fade_out_time)
    }

    pub fn get_layout(&self) -> Option<&Layout> {
        self.layout.as_ref()
    }

    pub fn get_eye_blink_parameter_count(&self) -> usize {
        self.find_group("EyeBlink").map_or(0, |g| g.ids.len())
    }

    pub fn get_eye_blink_parameter_id(&self, index: usize) -> Option<&str> {
        self.find_group("EyeBlink")
            .and_then(|g| g.ids.get(index))
            .map(|id| id.as_str())
    }

    pub fn get_lip_sync_parameter_count(&self) -> usize {
        self.find_group("LipSync").map_or(0, |g| g.ids.len())
    }

    pub fn get_lip_sync_parameter_id(&self, index: usize) -> Option<&str> {
        self.find_group("LipSync")
            .and_then(|g| g.ids.get(index))
            .map(|id| id.as_str())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_model3_json() {
        let model3 = ModelSetting::new("./test_file/test.model3.json").unwrap();
        let res = ModelSetting {
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
