use std::error::Error;
use std::fs;
use std::path::Path;

use crate::expression::json::*;
use crate::model::Model;
use crate::motion::amotion::*;
use crate::motion::queue::*;

#[derive(Debug)]
pub enum ExpBlendType {
    Add,
    Multiply,
    Overwrite,
}

#[derive(Debug)]
pub struct ExpParam {
    pub id: String,
    pub blend_type: ExpBlendType,
    pub value: f32,
}

#[derive(Debug)]
pub struct ExpValue {
    pub id: String,
    pub add_value: f32,
    pub mul_value: f32,
    pub ow_value: f32,
}

impl ExpValue {
    pub fn reset(&mut self, default_val: f32) {
        self.add_value = 0.0;
        self.mul_value = 1.0;
        self.ow_value = default_val;
    }
}

pub struct ExpMotion {
    pub base: MotionBase,
    pub fade_weight: f32,
    pub params: Vec<ExpParam>,
}

impl ExpMotion {
    pub fn from_path(base_dir: &str, path: &str) -> Result<Self, Box<dyn Error>> {
        let full_path = Path::new(base_dir).join(path);
        let data = fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read file {:?}: {}", full_path, e))?;

        let exp3: Exp3 = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse JSON ({:?}): {}", full_path, e))?;

        let mut base = MotionBase::new();
        base.fade_in_seconds = if exp3.fade_in_time <= 0.0 {
            1.0
        } else {
            exp3.fade_in_time
        };
        base.fade_out_seconds = if exp3.fade_out_time <= 0.0 {
            1.0
        } else {
            exp3.fade_out_time
        };
        base.is_loop = false;

        let params = exp3
            .parameters
            .into_iter()
            .map(|p| {
                let blend_type = match p.blend.as_str() {
                    "Add" => ExpBlendType::Add,
                    "Multiply" => ExpBlendType::Multiply,
                    "Overwrite" => ExpBlendType::Overwrite,
                    _ => ExpBlendType::Add,
                };

                ExpParam {
                    id: p.id,
                    blend_type,
                    value: p.value,
                }
            })
            .collect();

        Ok(Self {
            base,
            fade_weight: 0.0,
            params,
        })
    }

    pub fn do_update_parameters(
        &mut self,
        model: &mut Model,
        _user_time_s: f32,
        weight: f32,
        _qe: &mut MotionQueueEntry,
    ) {
        for param in &self.params {
            match param.blend_type {
                ExpBlendType::Add => {
                    model.add_parameter_value_by_id(&param.id, param.value, weight);
                }
                ExpBlendType::Multiply => {
                    model.multiply_parameter_value_by_id(&param.id, param.value, weight);
                }
                ExpBlendType::Overwrite => {
                    model.set_parameter_value_by_id(&param.id, param.value, weight);
                }
            }
        }
    }

    pub fn cal_exp_params(
        &mut self,
        model: &mut Model,
        user_time_s: f32,
        qe: &mut MotionQueueEntry,
        expression_parameter_values: &mut Vec<ExpValue>,
        expression_index: usize,
        fade_weight: f32,
    ) {
        if !qe.available {
            return;
        }

        self.fade_weight = self.update_fade_weight(qe, user_time_s);

        let default_additive = 0.0;
        let default_multiply = 1.0;

        for ep_val in expression_parameter_values {
            // TODO: Error handle
            let current_model_val = model.get_parameter_value_by_id(&ep_val.id);

            let target_config = self.params.iter().find(|p| p.id == ep_val.id);

            match target_config {
                None => {
                    if expression_index == 0 {
                        ep_val.reset(current_model_val);
                    } else {
                        ep_val.add_value =
                            self.cal_value(ep_val.add_value, default_additive, fade_weight);
                        ep_val.mul_value =
                            self.cal_value(ep_val.mul_value, default_multiply, fade_weight);
                        ep_val.ow_value =
                            self.cal_value(ep_val.ow_value, current_model_val, fade_weight);
                    }
                }
                Some(config) => {
                    let (target_add, target_mul, target_set) = match config.blend_type {
                        ExpBlendType::Add => (config.value, default_multiply, current_model_val),
                        ExpBlendType::Multiply => {
                            (default_additive, config.value, current_model_val)
                        }
                        ExpBlendType::Overwrite => {
                            (default_additive, default_multiply, config.value)
                        }
                    };

                    if expression_index == 0 {
                        ep_val.add_value =
                            self.cal_value(default_additive, target_add, fade_weight);
                        ep_val.mul_value =
                            self.cal_value(default_multiply, target_mul, fade_weight);
                        ep_val.ow_value =
                            self.cal_value(current_model_val, target_set, fade_weight);
                    //                        ep_val.add_value = target_add;
                    //                        ep_val.mul_value = target_mul;
                    //                        ep_val.ow_value = target_set;
                    } else {
                        ep_val.add_value =
                            self.cal_value(ep_val.add_value, target_add, fade_weight);
                        ep_val.mul_value =
                            self.cal_value(ep_val.mul_value, target_mul, fade_weight);
                        ep_val.ow_value = self.cal_value(ep_val.ow_value, target_set, fade_weight);
                    }
                }
            }
        }
    }

    pub fn cal_value(&mut self, source: f32, dest: f32, fade_weight: f32) -> f32 {
        source * (1. - fade_weight) + dest * fade_weight
    }
}

impl ACubismMotion for ExpMotion {
    fn to_exp_motion(&self) -> Option<&ExpMotion> {
        Some(&self)
    }
    fn to_exp_motion_mut(&mut self) -> Option<&mut ExpMotion> {
        Some(self)
    }

    fn base_mut(&mut self) -> &mut MotionBase {
        &mut self.base
    }

    fn base(&self) -> &MotionBase {
        &self.base
    }

    fn update_parameters(
        &mut self,
        model: &mut Model,
        qe: &mut MotionQueueEntry,
        user_time_s: f32,
    ) {
        if !qe.available || qe.finished {
            return;
        }
        self.setup_motion_queue_entry(qe, user_time_s);
        let fade_weight = self.update_fade_weight(qe, user_time_s);
        self.do_update_parameters(model, user_time_s, fade_weight, qe);
        if qe.end_time_seconds > 0. && qe.end_time_seconds < user_time_s {
            qe.finished = true;
        }
    }

    fn adjust_end_time(&self, qe: &mut MotionQueueEntry) {
        let duration = self.get_duration();
        qe.end_time_seconds = if duration <= 0. {
            -1.
        } else {
            qe.start_time_seconds + duration
        };
    }

    fn setup_motion_queue_entry(&self, entry: &mut MotionQueueEntry, user_time: f32) {
        if !entry.available || entry.finished || entry.started {
            return;
        }

        entry.started = true;
        entry.start_time_seconds = user_time - self.base.offset_seconds;
        entry.fade_in_start_time_seconds = user_time;

        if entry.end_time_seconds < 0.0 {
            self.adjust_end_time(entry);
        }
    }

    fn update_fade_weight(&self, entry: &mut MotionQueueEntry, user_time: f32) -> f32 {
        let mut fade_weight = self.base.weight;

        let fade_in = if self.base.fade_in_seconds <= 0.0 {
            1.0
        } else {
            get_easing_sine(
                (user_time - entry.fade_in_start_time_seconds) / self.base.fade_in_seconds,
            )
        };

        let fade_out = if self.base.fade_out_seconds <= 0.0 || entry.end_time_seconds < 0.0 {
            1.0
        } else {
            get_easing_sine((entry.end_time_seconds - user_time) / self.base.fade_out_seconds)
        };

        fade_weight = fade_weight * fade_in * fade_out;
        entry.set_state(user_time, fade_weight);

        fade_weight.clamp(0.0, 1.0)
    }

    fn get_fired_events(
        &mut self,
        _before_check_time_seconds: f32,
        _motion_time_seconds: f32,
    ) -> Vec<String> {
        vec![]
    }
}
