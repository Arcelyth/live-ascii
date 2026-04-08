use std::collections::HashMap;

use crate::model::Model;
use crate::tracker::*;

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

pub struct FaceController {
    current_values: HashMap<String, f32>,
    smooth_factor: f32, 
}

impl FaceController {
    pub fn new(smooth_factor: f32) -> Self {
        Self {
            current_values: HashMap::new(),
            smooth_factor,
        }
    }

    fn set_param_smoothed(&mut self, model: &mut Model, param_id: &str, target_value: f32) {
        let current = self.current_values.entry(param_id.to_string()).or_insert(target_value);
        
        *current = lerp(*current, target_value, self.smooth_factor);
        
        model.set_parameter_value_by_id(param_id, *current, 1.); 
    }

    pub fn update_parameters(&mut self, model: &mut Model, packet: &Packet) {
        let angle_x = packet.euler[1] * -50.0;
        let angle_y = packet.euler[0] * 50.0; 
        let angle_z = packet.euler[2] * -50.0;

        self.set_param_smoothed(model, "ParamAngleX", angle_x);
        self.set_param_smoothed(model, "ParamAngleY", angle_y);
        self.set_param_smoothed(model, "ParamAngleZ", angle_z);

        // Body
        self.set_param_smoothed(model, "ParamBodyAngleX", angle_x * 0.3);
        self.set_param_smoothed(model, "ParamBodyAngleY", angle_y * 0.3);
        self.set_param_smoothed(model, "ParamBodyAngleZ", angle_z * 0.3);

        // Eye 
        let eye_l_open = 1.0 - packet.eye_blink_left;
        let eye_r_open = 1.0 - packet.eye_blink_right;

        self.set_param_smoothed(model, "ParamEyeLOpen", eye_l_open);
        self.set_param_smoothed(model, "ParamEyeROpen", eye_r_open);

        // EyeBall
        self.set_param_smoothed(model, "ParamEyeBallX", packet.euler[1] * -1.5);
        self.set_param_smoothed(model, "ParamEyeBallY", packet.euler[0] * -1.5);

        // Mouth
        let mouth_open_y = packet.mouth_open * 1.5;
        let mouth_form = (packet.mouth_wide * 2.0) - 1.0; 

        self.set_param_smoothed(model, "ParamMouthOpenY", mouth_open_y.clamp(0.0, 1.2));
        self.set_param_smoothed(model, "ParamMouthForm", mouth_form.clamp(-1.0, 1.0));
    }
}
