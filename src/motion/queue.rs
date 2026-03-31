use crate::motion::json::*;
use crate::motion::amotion::*;

pub struct MotionQueueEntry {
    pub auto_delete: bool,
    pub motion: CubismMotion,
    pub available: bool,
    pub finished: bool,
    pub started: bool,
    pub start_time_seconds: f32,
    pub fade_in_start_time_seconds: f32,
    pub end_time_seconds: f32,
    pub state_time_seconds: f32,
    pub state_weight: f32,
    pub last_event_check_seconds: f32,
    pub fade_out_seconds: f32,
    pub is_triggered_fade_out: bool,
}

impl MotionQueueEntry {
    pub fn new(motion: CubismMotion) -> Self {
        Self {
            auto_delete: false,
            motion: motion,
            available: true,
            finished: false,
            started: false,
            start_time_seconds: -1.,
            fade_in_start_time_seconds: 0.,
            end_time_seconds: -1.,
            state_time_seconds: 0.,
            state_weight: 0.,
            last_event_check_seconds: 0.,
            fade_out_seconds: 0.,
            is_triggered_fade_out: false,
        }
    }

    pub fn set_fade_out(&mut self, fade_out_s: f32) {
        self.fade_out_seconds = fade_out_s;
        self.is_triggered_fade_out = true;
    }

    pub fn start_fade_out(&mut self, fade_out_s: f32, user_time_s: f32) {
        let new_end_time_s = user_time_s + fade_out_s;
        self.is_triggered_fade_out = true;
        if self.end_time_seconds < 0. || new_end_time_s < self.end_time_seconds {
            self.end_time_seconds = new_end_time_s;
        }
    }

    pub fn set_state(&mut self, time_s: f32, weight: f32) {
        self.state_time_seconds = time_s;
        self.state_weight = weight;
    }
}

pub struct MotionQueueManager {
    user_time_seconds: f32,
    motions: Vec<MotionQueueEntry>,
}

impl MotionQueueManager {
    pub fn new() -> Self {
        Self {
            user_time_seconds: 0.,
            motions: vec![],
        }
    }

    pub fn start_motion(&mut self, motion: CubismMotion, auto_delete: bool) {
        for entry in &mut self.motions {
            entry.set_fade_out(entry.motion.base.fade_out_seconds);
        }
        let mut m_entry = MotionQueueEntry::new(motion);
        m_entry.auto_delete = auto_delete;
        self.motions.push(m_entry);
    }

    // TODO:
    pub fn do_update_motion(&mut self, user_time_s: f32) {}

    pub fn is_all_finished(&mut self) -> bool {
        self.motions.iter().all(|entry| entry.finished)
    }

    pub fn is_finished(&self, handle: usize) -> bool {
        if let Some(entry) = self.motions.get(handle) {
            entry.finished
        } else {
            true
        }
    }

    pub fn stop_all_motions(&mut self) {
        self.motions.clear();
    }
}
