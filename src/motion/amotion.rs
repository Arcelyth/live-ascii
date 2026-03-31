use crate::motion::json::*;
use crate::motion::queue::*;

pub trait AMotion {

    
}

pub struct MotionBase {
    pub fade_in_seconds: f32,
    pub fade_out_seconds: f32,
    pub weight: f32,
    pub offset_seconds: f32,
    pub is_loop: bool,
    pub is_loop_fade_in: bool,
    pub previous_loop_state: bool,
    pub fired_event_values: Vec<String>,
    // TODO: Callback and CustomData
}

impl MotionBase {
    pub fn new() -> Self {
        Self {
            fade_in_seconds: -1.,
            fade_out_seconds: 1.,
            weight: 1.,
            offset_seconds: 0.,
            is_loop: false,
            is_loop_fade_in: true,
            previous_loop_state: false,
            fired_event_values: vec![],
        }
    }
} 

pub struct CubismMotion {
    pub base: MotionBase,
    source_frame_rate: f32,
    loop_duration_seconds: f32,
    motion_behavior: MotionBehavior,
    last_weight: f32,
    motion_data: MotionData,
    model_curve_id_eye_blink: Vec<String>,
    model_curve_id_lip_sync: Vec<String>,
    model_curve_id_opacity: Vec<String>,
    model_opacity: f32,
}

impl CubismMotion {
    pub fn new(motion_data: MotionData) -> Self {
        let base = MotionBase::new();
        Self {
            base,
            source_frame_rate: motion_data.fps,
            loop_duration_seconds: motion_data.duration,
            motion_behavior: MotionBehavior::MotionBehaviorV2,
            last_weight: 0.,
            motion_data,
            model_curve_id_eye_blink: vec![],
            model_curve_id_lip_sync: vec![],
            model_curve_id_opacity: vec![],
            model_opacity: 1.,
        }
    }

    pub fn get_duration(&self) -> f32{
        if self.base.is_loop {
            -1.
        } else {
            self.loop_duration_seconds
        }
    } 

    // TODO
    pub fn do_update_parameters(&mut self, user_time_seconds: f32, fade_weight: f32, motion_queue_e: &MotionQueueEntry) {

    }

    // TODO
    pub fn update_for_next_loop(&mut self, user_time_seconds: f32, time: f32, motion_queue_e: &MotionQueueEntry) {
    }

    pub fn set_fade_in_time(&mut self, id: String, value: f32) {
        for curve in &mut self.motion_data.curves {
            if id == curve.id {
                curve.fade_in_time = value;
                return
            }
        }
    }

    pub fn set_fade_out_time(&mut self, id: String, value: f32) {
        for curve in &mut self.motion_data.curves {
            if id == curve.id {
                curve.fade_out_time = value;
                return
            }
        }
    }

    pub fn get_fade_in_time(&mut self, id: String, value: f32) -> Option<f32> {
        for curve in &mut self.motion_data.curves {
            if id == curve.id {
                return Some(curve.fade_in_time);                
            }
        }
        None
    }

    pub fn get_fade_out_time(&mut self, id: String, value: f32) -> Option<f32> {
        for curve in &mut self.motion_data.curves {
            if id == curve.id {
                return Some(curve.fade_out_time);                
            }
        }
        None
    }

    pub fn set_effect_ids(&mut self, eye_blink_ids: Vec<String>, lip_sync_ids: Vec<String>) {
        self.model_curve_id_eye_blink = eye_blink_ids; 
        self.model_curve_id_lip_sync = lip_sync_ids; 
    }
    
    // TODO:
    pub fn get_fired_event(&mut self, before_time: f32, motion_time: f32) {
    }

    pub fn is_exist_model_opacity(&self) -> bool {
        for curve in &self.motion_data.curves {
            if curve.target_type != CurveTargetType::Model {
                continue;
            }
            if curve.id == "Opacity" {
                return true
            }
        }
        false
    } 

    pub fn get_model_opacity_index(&self) -> Option<usize> {
        for (i, curve) in self.motion_data.curves.iter().enumerate() {
            if curve.target_type != CurveTargetType::Model {
                continue;
            }
            if curve.id == "Opacity" {
                return Some(i)
            }
        }
        return None
    }

    // TODO
//    pub fn get_model_opacity_id(idx: usize) -> String {
//        let curve = self.motion_data.curves[idx]; 
//        if curve.target_type == CurveTargetType::Model {
//            if curve.id == "Opacity" {
//
//                return 
//            }
//        }
//    }

}

pub enum MotionBehavior {
    MotionBehaviorV1,
    MotionBehaviorV2
}

