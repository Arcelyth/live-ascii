use crate::model::*;
use crate::motion::amotion::*;
use crate::motion::json::*;

pub struct MotionQueueEntry {
    pub id: usize,
    pub auto_delete: bool,
    pub motion: Box<dyn ACubismMotion>,
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
    pub fn new(motion: Box<dyn ACubismMotion>, id: usize) -> Self {
        Self {
            id,
            auto_delete: false,
            motion,
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
    pub user_time_seconds: f32,
    pub motions: Vec<MotionQueueEntry>,
    pub event_callback: Option<Box<dyn Fn(&str)>>,
    pub id_counter: usize,
}

impl MotionQueueManager {
    pub fn new() -> Self {
        Self {
            user_time_seconds: 0.,
            motions: vec![],
            event_callback: None,
            id_counter: 0,
        }
    }

    pub fn start_motion<A: ACubismMotion + 'static>(
        &mut self,
        motion: A,
        auto_delete: bool,
    ) -> usize {
        for entry in &mut self.motions {
            entry.set_fade_out(entry.motion.base().fade_out_seconds);
        }
        let mut m_entry = MotionQueueEntry::new(Box::new(motion), self.id_counter);
        self.id_counter += 1;
        m_entry.auto_delete = auto_delete;
        self.motions.push(m_entry);
        self.id_counter - 1
    }

    pub fn do_update_motion(&mut self, model: &mut Model, user_time_s: f32) -> bool {
        let mut updated = false;
        let callback = &self.event_callback;

        let mut remove_items = vec![];
        for (i, entry) in self.motions.iter_mut().enumerate() {
            let entry_ptr = entry as *mut MotionQueueEntry;
            unsafe {
                entry
                    .motion
                    .update_parameters(model, &mut *entry_ptr, user_time_s);
            }
            updated = true;

            let before_s = entry.last_event_check_seconds - entry.start_time_seconds;
            let current_s = user_time_s - entry.start_time_seconds;

            let fired_events = entry.motion.get_fired_events(before_s, current_s);

            for event_name in fired_events {
                if let Some(cb) = callback {
                    cb(&event_name);
                }
            }

            entry.last_event_check_seconds = user_time_s;

            if entry.finished {
                remove_items.push(i);
            }
//            else {
//                if entry.is_triggered_fade_out {
//                    entry.start_fade_out(entry.fade_out_seconds, user_time_s);
//                }
//            }
        }

        for i in remove_items.into_iter().rev() {
            self.motions.remove(i);
        }
        updated
    }

    pub fn is_all_finished(&mut self) -> bool {
        self.motions.iter().all(|entry| entry.finished)
    }

    pub fn is_finished(&self, id: usize) -> bool {
        self.motions
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.finished)
            .unwrap_or(true)
    }

    pub fn stop_all_motions(&mut self) {
        self.motions.clear();
    }
}
