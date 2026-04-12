use crate::motion::amotion::*;
use crate::model::Model;
use crate::motion::queue::*;

#[derive(Debug)]
pub struct MotionManager {
    pub qm: MotionQueueManager,
    pub current_prior: usize,
    pub reserve_prior: usize,
}

impl MotionManager {
    pub fn new() -> Self {
        let qm = MotionQueueManager::new();
        Self {
            qm, 
            current_prior: 0,
            reserve_prior: 0,
        }
    }

    pub fn start_motion_priority(&mut self, motion: CubismMotion, auto_delete: bool, priority: usize) {
        if priority == self.reserve_prior {
            self.reserve_prior = 0;
        }
        self.current_prior = priority;
        self.qm.start_motion(motion, auto_delete);
    }

    pub fn update_motion(&mut self, model: &mut Model, delta_time_s: f32) -> bool {
        self.qm.user_time_seconds += delta_time_s;
        let updated = self.qm.do_update_motion(model, self.qm.user_time_seconds);
        if self.qm.is_all_finished() {
            self.current_prior = 0;
        }
        updated
    }

    pub fn reserve_motion(&mut self, priority: usize) -> bool {
        if priority <= self.reserve_prior || priority <= self.current_prior {
            false
        } else {
            self.reserve_prior = priority;
            true
        }
    }
}

