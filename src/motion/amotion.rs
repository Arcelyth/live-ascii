#![allow(dead_code)]

use std::f32;

use crate::model::*;
use crate::motion::json::*;
use crate::motion::queue::*;

pub trait ACubismMotion {
    fn base(&self) -> &MotionBase;
    fn base_mut(&mut self) -> &mut MotionBase;
    fn update_parameters(&mut self, model: &mut Model, qe: &mut MotionQueueEntry, user_time_s: f32);
    fn get_fired_events(
        &mut self,
        before_check_time_seconds: f32,
        motion_time_seconds: f32,
    ) -> Vec<String>;
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
            is_loop: true,
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

    pub fn get_duration(&self) -> f32 {
        if self.base.is_loop {
            -1.
        } else {
            self.loop_duration_seconds
        }
    }

    pub fn adjust_end_time(&self, qe: &mut MotionQueueEntry) {
        let duration = self.get_duration();
        qe.end_time_seconds = if duration <= 0. {
            -1.
        } else {
            qe.start_time_seconds + duration
        };
    }


    pub fn setup_motion_queue_entry(&self, entry: &mut MotionQueueEntry, user_time: f32) {
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

    pub fn update_fade_weight(&self, entry: &mut MotionQueueEntry, user_time: f32) -> f32 {
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

    pub fn do_update_parameters(
        &mut self,
        model: &mut Model,
        user_time_seconds: f32,
        fade_weight: f32,
        motion_queue_e: &mut MotionQueueEntry,
    ) {
        if let MotionBehavior::MotionBehaviorV2 = self.motion_behavior {
            if self.base.previous_loop_state != self.base.is_loop {
                // recalculate the end time
                self.adjust_end_time(motion_queue_e);
                self.base.previous_loop_state = self.base.is_loop;
            }
        }

        let mut time_offset_seconds = user_time_seconds - motion_queue_e.start_time_seconds;
        if time_offset_seconds < 0.0 {
            time_offset_seconds = 0.0;
        }

        let mut lip_sync_value = f32::MAX;
        let mut eye_blink_value = f32::MAX;

        const MAX_TARGET_SIZE: usize = 64;
        let mut lip_sync_flags: u64 = 0;
        let mut eye_blink_flags: u64 = 0;

        let tmp_fade_in = if self.base.fade_in_seconds <= 0.0 {
            1.0
        } else {
            get_easing_sine(
                (user_time_seconds - motion_queue_e.fade_in_start_time_seconds)
                    / self.base.fade_in_seconds,
            )
        };

        let tmp_fade_out = if self.base.fade_out_seconds <= 0.0
            || motion_queue_e.end_time_seconds < 0.0
        {
            1.0
        } else {
            get_easing_sine(
                (motion_queue_e.end_time_seconds - user_time_seconds) / self.base.fade_out_seconds,
            )
        };

        let mut time = time_offset_seconds;
        let mut duration = self.motion_data.duration;
        let is_correction =
            matches!(self.motion_behavior, MotionBehavior::MotionBehaviorV2) && self.base.is_loop;

        if self.base.is_loop {
            if let MotionBehavior::MotionBehaviorV2 = self.motion_behavior {
                duration += 1.0 / self.source_frame_rate;
            }
            while time > duration {
                time -= duration;
            }
        }

        for curve in &self.motion_data.curves {
            let value = evaluate_curve(&self.motion_data, curve, time, is_correction, duration);

            match curve.target_type {
                CurveTargetType::Model => {
                    if curve.id == "EyeBlink" {
                        eye_blink_value = value;
                    } else if curve.id == "LipSync" {
                        lip_sync_value = value;
                    } else if curve.id == "Opacity" {
                        self.model_opacity = value;
                        model.model_opacity = self.model_opacity;
                    }
                }
                CurveTargetType::Parameter => {
                    let parameter_index = model.get_parameter_index(&curve.id);
                    let parameter_index = if let Some(idx) = parameter_index {
                        idx
                    } else {
                        continue;
                    };

                    let source_value = model.get_parameter_value(parameter_index);
                    let mut current_value = value;

                    if eye_blink_value != f32::MAX {
                        if let Some(pos) = self
                            .model_curve_id_eye_blink
                            .iter()
                            .position(|id| id == &curve.id)
                        {
                            if pos < MAX_TARGET_SIZE {
                                current_value *= eye_blink_value;
                                eye_blink_flags |= 1 << pos;
                            }
                        }
                    }

                    if lip_sync_value != f32::MAX {
                        if let Some(pos) = self
                            .model_curve_id_lip_sync
                            .iter()
                            .position(|id| id == &curve.id)
                        {
                            if pos < MAX_TARGET_SIZE {
                                current_value += lip_sync_value;
                                lip_sync_flags |= 1 << pos;
                            }
                        }
                    }

                    if model.is_repeat(parameter_index) {
                        current_value =
                            model.get_parameter_repeat_value(parameter_index, current_value);
                    }

                    let final_value;
                    if curve.fade_in_time < 0.0 && curve.fade_out_time < 0.0 {
                        final_value = source_value + (current_value - source_value) * fade_weight;
                    } else {
                        let fin = if curve.fade_in_time < 0.0 {
                            tmp_fade_in
                        } else if curve.fade_in_time == 0.0 {
                            1.0
                        } else {
                            get_easing_sine(
                                (user_time_seconds - motion_queue_e.fade_in_start_time_seconds)
                                    / curve.fade_in_time,
                            )
                        };

                        let fout = if curve.fade_out_time < 0.0 {
                            tmp_fade_out
                        } else if curve.fade_out_time == 0.0
                            || motion_queue_e.end_time_seconds < 0.0
                        {
                            1.0
                        } else {
                            get_easing_sine(
                                (motion_queue_e.end_time_seconds - user_time_seconds)
                                    / curve.fade_out_time,
                            )
                        };

                        let param_weight = self.base.weight * fin * fout;
                        final_value = source_value + (current_value - source_value) * param_weight;
                    }

                    model.set_parameter_value(parameter_index, final_value, 1.);
                }
                CurveTargetType::PartOpacity => {
                    let parameter_index = model.get_parameter_index(&curve.id);
                    if let Some(idx) = parameter_index {
                        model.set_parameter_value(idx, value, 1.);
                    }
                }
            }
        }

        if eye_blink_value != f32::MAX {
            for (i, id) in self
                .model_curve_id_eye_blink
                .iter()
                .enumerate()
                .take(MAX_TARGET_SIZE)
            {
                if (eye_blink_flags >> i) & 0x01 != 0 {
                    continue;
                }
                let source_value = if let Some(v) = model.get_parameter_value_by_id(id) {
                    v
                } else {
                    continue;
                };
                let v = source_value + (eye_blink_value - source_value) * fade_weight;
                model.set_parameter_value_by_id(id, v, 1.);
            }
        }

        if lip_sync_value != f32::MAX {
            for (i, id) in self
                .model_curve_id_lip_sync
                .iter()
                .enumerate()
                .take(MAX_TARGET_SIZE)
            {
                if (lip_sync_flags >> i) & 0x01 != 0 {
                    continue;
                }
                let source_value = if let Some(v) = model.get_parameter_value_by_id(id) {
                    v
                } else {
                    continue;
                };

                let v = source_value + (lip_sync_value - source_value) * fade_weight;
                model.set_parameter_value_by_id(id, v, 1.);
            }
        }

        if time_offset_seconds >= duration {
            if self.base.is_loop {
                self.update_for_next_loop(user_time_seconds, time, motion_queue_e);
            } else {
                motion_queue_e.finished = true;
            }
        }

        self.last_weight = fade_weight;
    }

    pub fn update_for_next_loop(
        &mut self,
        user_time_seconds: f32,
        time: f32,
        motion_queue_e: &mut MotionQueueEntry,
    ) {
        match self.motion_behavior {
            MotionBehavior::MotionBehaviorV1 => {
                motion_queue_e.start_time_seconds = user_time_seconds;
                if self.base.is_loop_fade_in {
                    motion_queue_e.fade_in_start_time_seconds = user_time_seconds;
                }
            }
            MotionBehavior::MotionBehaviorV2 => {
                motion_queue_e.start_time_seconds = user_time_seconds - time;
                if self.base.is_loop_fade_in {
                    motion_queue_e.fade_in_start_time_seconds = user_time_seconds - time;
                }
            }
        }
    }
    pub fn set_fade_in_time(&mut self, id: String, value: f32) {
        for curve in &mut self.motion_data.curves {
            if id == curve.id {
                curve.fade_in_time = value;
                return;
            }
        }
    }

    pub fn set_fade_out_time(&mut self, id: String, value: f32) {
        for curve in &mut self.motion_data.curves {
            if id == curve.id {
                curve.fade_out_time = value;
                return;
            }
        }
    }

    pub fn get_fade_in_time(&mut self, id: String) -> Option<f32> {
        for curve in &mut self.motion_data.curves {
            if id == curve.id {
                return Some(curve.fade_in_time);
            }
        }
        None
    }

    pub fn get_fade_out_time(&mut self, id: String) -> Option<f32> {
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

    pub fn is_exist_model_opacity(&self) -> bool {
        for curve in &self.motion_data.curves {
            if curve.target_type != CurveTargetType::Model {
                continue;
            }
            if curve.id == "Opacity" {
                return true;
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
                return Some(i);
            }
        }
        return None;
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

impl ACubismMotion for CubismMotion {

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

    fn get_fired_events(
        &mut self,
        before_check_time_seconds: f32,
        motion_time_seconds: f32,
    ) -> Vec<String> {
        self.base.fired_event_values.clear();

        for event in &self.motion_data.events {
            if event.time > before_check_time_seconds && event.time <= motion_time_seconds {
                self.base.fired_event_values.push(event.value.clone());
            }
        }

        self.base.fired_event_values.clone()
    }
}

pub enum MotionBehavior {
    MotionBehaviorV1,
    MotionBehaviorV2,
}

pub fn correct_end_point(
    motion_data: &MotionData,
    last_segment_index: usize,
    first_point_index: usize,
    last_point_index: usize,
    time: f32,
    end_time: f32,
) -> f32 {
    let last_segment = &motion_data.segments[last_segment_index];
    let first_point = &motion_data.points[first_point_index];
    let last_point = &motion_data.points[last_point_index];

    let t = (time - last_point.time) / (end_time - last_point.time);
    last_point.value + (first_point.value - last_point.value) * t
}

pub fn evaluate_curve(
    motion_data: &MotionData,
    curve: &MotionCurve,
    time: f32,
    is_correction: bool,
    end_time: f32,
) -> f32 {
    let mut target: i32 = -1;
    let total_segment_count = curve.base_segment_index + curve.segment_count;
    let mut point_position = 0;

    for i in curve.base_segment_index..total_segment_count {
        let segment = &motion_data.segments[i];

        point_position = segment.base_point_index
            + if segment.segment_type == MotionSegmentType::Bezier {
                3
            } else {
                1
            };

        if motion_data.points[point_position].time > time {
            target = i as i32;
            break;
        }
    }

    if target == -1 {
        if is_correction && time < end_time {
            return correct_end_point(
                motion_data,
                total_segment_count - 1,
                motion_data.segments[curve.base_segment_index].base_point_index,
                point_position,
                time,
                end_time,
            );
        }
        return motion_data.points[point_position].value;
    }

    let segment = &motion_data.segments[target as usize];

    evaluate_segment(segment, &motion_data.points, time)
}

fn evaluate_segment(segment: &MotionSegment, points: &[SegmentPoint], time: f32) -> f32 {
    let base_idx = segment.base_point_index;

    match segment.segment_type {
        MotionSegmentType::Linear => linear_evaluate(points[base_idx], points[base_idx + 1], time),
        MotionSegmentType::Bezier => bezier_evaluate(
            points[base_idx],
            points[base_idx + 1],
            points[base_idx + 2],
            points[base_idx + 3],
            time,
        ),
        MotionSegmentType::Stepped => points[base_idx].value,
        MotionSegmentType::InverseStepped => points[base_idx + 1].value,
    }
}

pub fn linear_evaluate(p0: SegmentPoint, p1: SegmentPoint, time: f32) -> f32 {
    let t = ((time - p0.time) / (p1.time - p0.time)).clamp(0.0, 1.0);
    p0.value + ((p1.value - p0.value) * t)
}

pub fn bezier_evaluate(
    p0: SegmentPoint,
    p1: SegmentPoint,
    p2: SegmentPoint,
    p3: SegmentPoint,
    time: f32,
) -> f32 {
    let mut t_min = 0.0;
    let mut t_max = 1.0;
    let mut t = 0.0;

    for _ in 0..20 {
        t = (t_min + t_max) / 2.0;
        let mt = 1.0 - t;

        let current_time = mt * mt * mt * p0.time
            + 3.0 * mt * mt * t * p1.time
            + 3.0 * mt * t * t * p2.time
            + t * t * t * p3.time;

        if current_time < time {
            t_min = t;
        } else {
            t_max = t;
        }
    }

    let mt = 1.0 - t;
    mt * mt * mt * p0.value
        + 3.0 * mt * mt * t * p1.value
        + 3.0 * mt * t * t * p2.value
        + t * t * t * p3.value
}

fn get_easing_sine(v: f32) -> f32 {
    (v * f32::consts::PI / 2.0).sin()
}
