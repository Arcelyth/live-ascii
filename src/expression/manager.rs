#![allow(dead_code)]

use crate::model::Model;
use crate::motion::queue::*;

pub struct ExpressionParameterValue {
    pub parameter_id: String,
    pub additive_value: f32,
    pub multiply_value: f32,
    pub overwrite_value: f32,
}

pub struct ExpressionManager<'m> {
    pub qm: MotionQueueManager<'m>,
    current_prior: usize,
    reserve_prior: usize,
    expression_parameters: Vec<ExpressionParameterValue>,
    fade_weights: Vec<f32>,
}

impl<'m> ExpressionManager<'m> {
    pub fn new() -> Self {
        Self {
            qm: MotionQueueManager::new(),
            current_prior: 0,
            reserve_prior: 0,
            expression_parameters: vec![],
            fade_weights: vec![],
        }
    }
    // TODO
    pub fn update_motion(&mut self, model: &mut Model, delta_time: f32) {
   
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
