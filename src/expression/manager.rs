#![allow(dead_code)]
use crate::expression::exp::*;
use crate::model::Model;
use crate::motion::amotion::*;
use crate::motion::queue::*;

#[derive(Debug)]
pub struct ExpressionManager {
    pub qm: MotionQueueManager,
    current_prior: usize,
    reserve_prior: usize,
    expression_parameters: Vec<ExpValue>,
    fade_weights: Vec<f32>,
}

impl ExpressionManager {
    pub fn new() -> Self {
        Self {
            qm: MotionQueueManager::new(),
            current_prior: 0,
            reserve_prior: 0,
            expression_parameters: vec![],
            fade_weights: vec![],
        }
    }

    pub fn update_motion(&mut self, model: &mut Model, delta_time: f32) -> bool {
        self.qm.user_time_seconds += delta_time;
        let mut updated = false;

        while self.fade_weights.len() < self.qm.motions.len() {
            self.fade_weights.push(0.0);
        }

        let mut expression_weight = 0.0;
        let mut expression_index = 0;

        let user_time = self.qm.user_time_seconds;

        for i in 0..self.qm.motions.len() {
            let motion_entry = &mut self.qm.motions[i];
            {
                let MotionQueueEntry {
                    motion, state: qe, ..
                } = motion_entry;
                let Some(exp_motion) = motion.to_exp_motion_mut() else {
                    continue;
                };
                if !qe.available || qe.finished {
                    continue;
                }

                if qe.available {
                    for param in &exp_motion.params {
                        if !self.expression_parameters.iter().any(|v| v.id == param.id) {
                            self.expression_parameters.push(ExpValue {
                                id: param.id.clone(),
                                add_value: 0.0,
                                mul_value: 1.0,
                                ow_value: model.get_parameter_value_by_id(&param.id),
                            });
                        }
                    }
                }

                if qe.is_triggered_fade_out {
                    qe.start_fade_out(qe.fade_out_seconds, user_time);
                }
                exp_motion.setup_motion_queue_entry(qe, user_time);

                let fade_weight = exp_motion.update_fade_weight(qe, user_time);
                self.fade_weights[i] = fade_weight;

                exp_motion.cal_exp_params(
                    model,
                    user_time,
                    qe,
                    &mut self.expression_parameters,
                    expression_index,
                    fade_weight,
                );
                let fade_in_time = exp_motion.base.fade_in_seconds;
                expression_weight += if fade_in_time <= 0.0 {
                    1.0
                } else {
                    let t = (user_time - qe.fade_in_start_time_seconds) / fade_in_time;
                    0.5 - 0.5 * ((t.clamp(0.0, 1.0) * std::f32::consts::PI).cos())
                };

                if qe.end_time_seconds >= 0.0 && user_time >= qe.end_time_seconds {
                    qe.finished = true;
                }
                updated = true;
                expression_index += 1;
            }
        }

        for i in (0..self.qm.motions.len()).rev() {
            if self.qm.motions[i].finished {
                self.qm.motions.remove(i);
                self.fade_weights.remove(i);
            }
        }
        self.fade_weights.truncate(self.qm.motions.len());

        let final_weight = expression_weight.min(1.0);

        for param_val in &mut self.expression_parameters {
            let value_to_set = (param_val.ow_value + param_val.add_value) * param_val.mul_value;

            model.set_parameter_value_by_id(&param_val.id, value_to_set, final_weight);

            param_val.add_value = 0.0;
            param_val.mul_value = 1.0;
        }

        updated
    }

    pub fn get_fade_weight(&self, index: usize) -> Option<f32> {
        if self.fade_weights.len() <= index {
            eprintln!(
                "Failed to get the fade weight value. The element at that index does not exist."
            );
            return None;
        }
        Some(self.fade_weights[index])
    }

    pub fn set_fade_weight(&mut self, index: usize, exp_fade_weight: f32) {
        if self.fade_weights.len() <= index {
            eprintln!(
                "Failed to get the fade weight value. The element at that index does not exist."
            );
            return;
        }
        self.fade_weights[index] = exp_fade_weight;
    }
}
