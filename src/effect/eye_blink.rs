use crate::model::Model;
use crate::model_setting::*;

pub enum EyeState {
    First,    // Initial state
    Interval, // State of not blinking
    Closing,  // State of closing the eyelids
    Closed,   // State where the eyelids are closed
    Opening,  // State of opening the eyelids
}

pub struct EyeBlink {
    blink_state: EyeState,
    param_ids: Vec<String>,
    next_blink_time: f32,
    state_start_time_seconds: f32,
    blink_interval_seconds: f32,
    closing_seconds: f32,
    closed_seconds: f32,
    opening_seconds: f32,
    user_time_seconds: f32,
}

impl EyeBlink {
    pub fn new(model_setting: &mut ModelSetting) -> Self {
        let mut param_ids = vec![];
        for i in 0..model_setting.get_eye_blink_parameter_count() {
            // TODO: Error handle
            param_ids.push(
                model_setting
                    .get_eye_blink_parameter_id(i)
                    .unwrap()
                    .to_string(),
            );
        }
        Self {
            blink_state: EyeState::First,
            param_ids,
            next_blink_time: 0.,
            state_start_time_seconds: 0.,
            blink_interval_seconds: 4.,
            closing_seconds: 0.1,
            closed_seconds: 0.05,
            opening_seconds: 0.15,
            user_time_seconds: 0.,
        }
    }

    fn determine_next_blinking_timing(&self) -> f32 {
        let r: f32 = rand::random::<f32>();
        self.user_time_seconds + (r * (2.0 * self.blink_interval_seconds - 1.0))
    }

    pub fn set_blinking_interval(&mut self, blinking_interval: f32) {
        self.blink_interval_seconds = blinking_interval;
    }

    pub fn set_blinking_settings(&mut self, closing: f32, closed: f32, opening: f32) {
        self.closing_seconds = closing;
        self.closed_seconds = closed;
        self.opening_seconds = opening;
    }

    pub fn set_parameter_ids(&mut self, parameter_ids: Vec<String>) {
        self.param_ids = parameter_ids;
    }

    pub fn get_parameter_ids(&self) -> &Vec<String> {
        &self.param_ids
    }

    pub fn update_parameters(&mut self, model: &mut Model, delta_time_seconds: f32) {
        self.user_time_seconds += delta_time_seconds;
        let mut parameter_value: f32;
        let mut t: f32;

        match self.blink_state {
            EyeState::Closing => {
                t = (self.user_time_seconds - self.state_start_time_seconds) / self.closing_seconds;

                if t >= 1.0 {
                    t = 1.0;
                    self.blink_state = EyeState::Closed;
                    self.state_start_time_seconds = self.user_time_seconds;
                }

                parameter_value = 1.0 - t;
            }
            EyeState::Closed => {
                t = (self.user_time_seconds - self.state_start_time_seconds) / self.closed_seconds;

                if t >= 1.0 {
                    self.blink_state = EyeState::Opening;
                    self.state_start_time_seconds = self.user_time_seconds;
                }

                parameter_value = 0.0;
            }
            EyeState::Opening => {
                t = (self.user_time_seconds - self.state_start_time_seconds) / self.opening_seconds;

                if t >= 1.0 {
                    t = 1.0;
                    self.blink_state = EyeState::Interval;
                    self.next_blink_time = self.determine_next_blinking_timing();
                }

                parameter_value = t;
            }
            EyeState::Interval => {
                if self.next_blink_time < self.user_time_seconds {
                    self.blink_state = EyeState::Closing;
                    self.state_start_time_seconds = self.user_time_seconds;
                }

                parameter_value = 1.0;
            }
            EyeState::First => {
                self.blink_state = EyeState::Interval;
                self.next_blink_time = self.determine_next_blinking_timing();
                parameter_value = 1.0;
            }
        }

        let close_if_zero = true;
        if !close_if_zero {
            parameter_value = -parameter_value;
        }

        for id in &self.param_ids {
            model.set_parameter_value_by_id(id, parameter_value, 1.0);
        }
    }
}
