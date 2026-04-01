use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::model::Model;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Pose3 {
    pub res_type: Option<String>,
    pub fade_in_time: Option<f32>,
    #[serde(default)]
    pub groups: Vec<Vec<PartItem>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PartItem {
    pub id: String,
    #[serde(default)]
    pub link: Vec<String>,
}

pub struct PartData {
    pub part_id: String,
    pub param_index: Option<usize>,
    pub part_index: Option<usize>,
    pub link: Vec<PartData>,
}

impl PartData {
    pub fn initialize(&mut self, model: &mut Model) {
        self.param_index = model.get_parameter_index(&self.part_id);
        self.part_index = Some(model.get_part_index(&self.part_id));

        if let Some(p_idx) = self.param_index {
            model.set_parameter_value(p_idx, 1.0, 1.0);
        }

        for link_item in &mut self.link {
            link_item.part_index = Some(model.get_part_index(&link_item.part_id));
            link_item.param_index = model.get_parameter_index(&link_item.part_id);
            if let Some(lp_idx) = link_item.param_index {
                model.set_parameter_value(lp_idx, 1.0, 1.0);
            }
        }
    }
}

pub struct Pose {
    pub part_groups: Vec<PartData>,
    pub part_group_counts: Vec<usize>,
    pub fade_time_seconds: f32,
}

impl Pose {
    const DEFAULT_FADE_IN_SECONDS: f32 = 0.5;
    const EPSILON: f32 = 0.001;
    const PHI: f32 = 0.5;
    const BACK_OPACITY_THRESHOLD: f32 = 0.15;

    pub fn new(base_dir: &str, path: &str) -> Result<Self, Box<dyn Error>> {
        let full_path = Path::new(base_dir).join(path);
        let data = fs::read_to_string(&full_path)?;
        let p3: Pose3 = serde_json::from_str(&data)?;

        let mut part_groups = Vec::new();
        let mut part_group_counts = Vec::new();

        for group in p3.groups {
            let mut count = 0;
            for item in group {
                let part_data = PartData {
                    part_id: item.id,
                    param_index: None,
                    part_index: None,
                    link: item
                        .link
                        .into_iter()
                        .map(|link_id| PartData {
                            part_id: link_id,
                            param_index: None,
                            part_index: None,
                            link: Vec::new(),
                        })
                        .collect(),
                };
                part_groups.push(part_data);
                count += 1;
            }
            part_group_counts.push(count);
        }

        Ok(Self {
            part_groups,
            part_group_counts,
            fade_time_seconds: p3
                .fade_in_time
                .unwrap_or(Self::DEFAULT_FADE_IN_SECONDS)
                .max(0.0),
        })
    }

    pub fn reset(&mut self, model: &mut Model) {
        let mut begin_index = 0;
        let part_groups = &mut self.part_groups;

        for &count in &self.part_group_counts {
            for j in begin_index..(begin_index + count) {
                let data = &mut part_groups[j];

                data.initialize(model);
                let parts_index = data.part_index;
                let param_index = data.param_index;

                let value = if j == begin_index { 1.0 } else { 0.0 };

                if let Some(pi) = parts_index {
                    model.set_part_opacity(pi, value);
                }
                if let Some(pai) = param_index {
                    model.set_parameter_value(pai, value, 1.0);
                }
            }
            begin_index += count;
        }
    }

    pub fn update_parameters(&mut self, model: &mut Model, delta_time_seconds: f32) {
        let dt = delta_time_seconds.max(0.0);
        let mut begin_index = 0;

        for i in 0..self.part_group_counts.len() {
            let count = self.part_group_counts[i];

            self.do_fade(model, dt, begin_index, count);

            begin_index += count;
        }

        self.copy_part_opacities(model);
    }

    fn do_fade(
        &mut self,
        model: &mut Model,
        delta_time_seconds: f32,
        begin_index: usize,
        part_group_count: usize,
    ) {
        let mut visible_part_index: i32 = -1;
        let mut new_opacity: f32 = 1.0;

        for i in begin_index..(begin_index + part_group_count) {
            let part_data = &self.part_groups[i];

            if let Some(param_idx) = part_data.param_index {
                if model.get_parameter_value(param_idx) > Self::EPSILON {
                    if visible_part_index >= 0 {
                        break;
                    }

                    visible_part_index = i as i32;

                    if self.fade_time_seconds <= 0.0 {
                        new_opacity = 1.0;
                    } else if let Some(part_idx) = part_data.part_index {
                        new_opacity = model.get_part_opacity(part_idx);
                        new_opacity += delta_time_seconds / self.fade_time_seconds;
                        if new_opacity > 1.0 {
                            new_opacity = 1.0;
                        }
                    }
                }
            }
        }

        if visible_part_index < 0 {
            visible_part_index = 0;
            new_opacity = 1.0;
        }

        for i in begin_index..(begin_index + part_group_count) {
            let part_idx = match self.part_groups[i].part_index {
                Some(idx) => idx,
                None => continue,
            };

            if visible_part_index == i as i32 {
                model.set_part_opacity(part_idx, new_opacity);
            } else {
                let current_opacity = model.get_part_opacity(part_idx);
                let mut a1: f32;

                if new_opacity < Self::PHI {
                    a1 = new_opacity * (Self::PHI - 1.0) / Self::PHI + 1.0;
                } else {
                    a1 = (1.0 - new_opacity) * Self::PHI / (1.0 - Self::PHI);
                }

                let back_opacity = (1.0 - a1) * (1.0 - new_opacity);
                if back_opacity > Self::BACK_OPACITY_THRESHOLD {
                    a1 = 1.0 - Self::BACK_OPACITY_THRESHOLD / (1.0 - new_opacity);
                }

                if current_opacity > a1 {
                    model.set_part_opacity(part_idx, a1);
                }
            }
        }
    }

    pub fn copy_part_opacities(&self, model: &mut Model) {
        for group in &self.part_groups {
            if group.link.is_empty() {
                continue;
            }

            if let Some(part_idx) = group.part_index {
                let opacity = model.get_part_opacity(part_idx);

                for link_part in &group.link {
                    if let Some(link_idx) = link_part.part_index {
                        model.set_part_opacity(link_idx, opacity);
                    }
                }
            }
        }
    }
}


