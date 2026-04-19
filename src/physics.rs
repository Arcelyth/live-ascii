use std::slice;
use std::f32::consts::PI;

use glam::Vec2;

use crate::ffi::*;
use crate::model::Model;
use crate::physics::json::*;

pub mod json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsTargetType {
    Parameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsSource {
    X,
    Y,
    Angle,
}

#[derive(Debug, Clone)]
pub struct PhysicsParameter {
    pub id: String,
    pub target_type: PhysicsTargetType,
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsNormalization {
    pub minimum: f32,
    pub maximum: f32,
    pub default: f32,
}

impl PhysicsNormalization {
    pub fn new(minimum: f32, maximum: f32, default: f32) -> Self {
        Self {
            minimum,
            maximum,
            default,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsParticle {
    pub initial_position: Vec2,
    pub mobility: f32,
    pub delay: f32,
    pub acceleration: f32,
    pub radius: f32,
    pub position: Vec2,
    pub last_position: Vec2,
    pub last_gravity: Vec2,
    pub force: Vec2,
    pub velocity: Vec2,
}

impl PhysicsParticle {
    pub fn new(mobility: f32, delay: f32, acceleration: f32, radius: f32, position: Vec2) -> Self {
        Self {
            initial_position: Vec2::new(0., 0.),
            mobility,
            delay,
            acceleration,
            radius,
            position,
            last_position: Vec2::new(0., 0.),
            last_gravity: Vec2::new(0., 0.),
            force: Vec2::new(0., 0.),
            velocity: Vec2::new(0., 0.),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicsSubRig {
    pub input_count: usize,
    pub output_count: usize,
    pub particle_count: usize,
    pub base_input_index: usize,
    pub base_output_index: usize,
    pub base_particle_index: usize,
    pub normalization_position: PhysicsNormalization,
    pub normalization_angle: PhysicsNormalization,
}

impl PhysicsSubRig {
    pub fn new(
        input_count: usize,
        output_count: usize,
        particle_count: usize,
        base_input_index: usize,
        base_output_index: usize,
        base_particle_index: usize,
        normalization_position: PhysicsNormalization,
        normalization_angle: PhysicsNormalization,
    ) -> Self {
        Self {
            input_count,
            output_count,
            particle_count,
            base_input_index,
            base_output_index,
            base_particle_index,
            normalization_position,
            normalization_angle,
        }
    }
}

pub type NormalizedPhysicsParameterValueGetter = fn(
    target_translation: &mut Vec2,
    target_angle: &mut f32,
    value: f32,
    parameter_minimum_value: f32,
    parameter_maximum_value: f32,
    parameter_default_value: f32,
    normalization_position: &PhysicsNormalization,
    normalization_angle: &PhysicsNormalization,
    is_inverted: bool,
    weight: f32,
);

pub type PhysicsValueGetter = fn(
    translation: Vec2,
    particles: &[PhysicsParticle],
    particle_index: usize,
    is_inverted: bool,
    parent_gravity: Vec2,
) -> f32;

pub type PhysicsScaleGetter = fn(translation_scale: Vec2, angle_scale: f32) -> f32;

#[derive(Clone)]
pub struct PhysicsInput {
    pub source: PhysicsParameter,
    pub source_parameter_index: i32,
    pub weight: f32,
    pub kind: PhysicsSource, // type
    pub reflect: bool,
    pub get_normalized_parameter_value: NormalizedPhysicsParameterValueGetter,
}

impl PhysicsInput {
    pub fn new(
        source: PhysicsParameter,
        source_parameter_index: i32,
        weight: f32,
        kind: PhysicsSource,
        reflect: bool,
        get_normalized_parameter_value: NormalizedPhysicsParameterValueGetter,
    ) -> Self {
        Self {
            source,
            source_parameter_index,
            weight,
            kind,
            reflect,
            get_normalized_parameter_value,
        }
    }
}

#[derive(Clone)]
pub struct PhysicsOutput {
    pub destination: PhysicsParameter,
    pub destination_parameter_index: i32,
    pub vertex_index: i32,
    pub translation_scale: Vec2,
    pub angle_scale: f32,
    pub weight: f32,
    pub kind: PhysicsSource, // type
    pub reflect: bool,
    pub value_below_minimum: f32,
    pub value_exceeded_maximum: f32,
    pub get_value: PhysicsValueGetter,
    pub get_scale: PhysicsScaleGetter,
}

impl PhysicsOutput {
    pub fn new(
        destination: PhysicsParameter,
        destination_parameter_index: i32,
        vertex_index: i32,
        angle_scale: f32,
        weight: f32,
        kind: PhysicsSource,
        reflect: bool,
        get_value: PhysicsValueGetter,
        get_scale: PhysicsScaleGetter,
    ) -> Self {
        Self {
            destination,
            destination_parameter_index,
            vertex_index,
            translation_scale: Vec2::new(0., 0.),
            angle_scale,
            weight,
            kind,
            reflect,
            value_below_minimum: 0.,
            value_exceeded_maximum: 0.,
            get_value,
            get_scale,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    pub gravity: Vec2,
    pub wind: Vec2,
}

impl Options {
    pub fn new() -> Self {
        Self {
            gravity: Vec2::new(0., -1.),
            wind: Vec2::new(0., 0.),
        }
    }
}

#[derive(Clone)]
pub struct PhysicsRig {
    pub sub_rig_count: usize,
    pub settings: Vec<PhysicsSubRig>,
    pub inputs: Vec<PhysicsInput>,
    pub outputs: Vec<PhysicsOutput>,
    pub particles: Vec<PhysicsParticle>,
    pub gravity: Vec2,
    pub wind: Vec2,
    pub fps: usize,
}

impl PhysicsRig {
    pub fn new(
        sub_rig_count: usize,
        settings: Vec<PhysicsSubRig>,
        inputs: Vec<PhysicsInput>,
        outputs: Vec<PhysicsOutput>,
        particles: Vec<PhysicsParticle>,
        gravity: Vec2,
        wind: Vec2,
        fps: usize,
    ) -> Self {
        Self {
            sub_rig_count,
            settings,
            inputs,
            outputs,
            particles,
            gravity,
            wind,
            fps,
        }
    }
}

#[derive(Clone)]
pub struct Physics {
    pub options: Options,
    pub physics_rig: PhysicsRig,
    pub current_rig_outputs: Vec<Vec<f32>>,
    pub previous_rig_outputs: Vec<Vec<f32>>,
    pub current_remain_time: f32,
    pub parameter_caches: Vec<f32>,
    pub parameter_input_caches: Vec<f32>,
}

impl Physics {
    const MAX_DELTA_TIME: f32 = 0.5;
    const MOVEMENT_THRESHOLD: f32 = 0.01;
    const AIR_RESISTANCE: f32 = 1.0;
    const MAX_WEIGHT: f32 = 100.;
    pub fn new(physics_rig: PhysicsRig) -> Self {
        Self {
            options: Options::new(),
            physics_rig,
            current_remain_time: 0.,
            current_rig_outputs: vec![],
            previous_rig_outputs: vec![],
            parameter_caches: vec![],
            parameter_input_caches: vec![],
        }
    }

    pub fn from_json(&self, json: PhysicsJson) -> Self {
        let forces = json.meta.effective_forces;
        let gravity = Vec2::new(forces.gravity.x, forces.gravity.y);
        let wind = Vec2::new(forces.wind.x, forces.wind.y);
        let sub_rig_count = json.meta.setting_count;
        let fps = json.meta.fps;
        let mut sub_rigs = vec![];
        let mut inputs = vec![];
        let mut outputs = vec![];
        let mut particles = vec![];
        for setting in json.settings {
            let setting_pos = setting.normalization.position;
            let setting_angle = setting.normalization.angle;
            let norm_pos = PhysicsNormalization::new(
                setting_pos.minimum,
                setting_pos.maximum,
                setting_pos.default,
            );
            let norm_angle = PhysicsNormalization::new(
                setting_angle.minimum,
                setting_angle.maximum,
                setting_angle.default,
            );

            // Inputs
            for input in setting.input {
                let get_norm_para_v: NormalizedPhysicsParameterValueGetter;
                let kind;
                (kind, get_norm_para_v) = match input.kind.as_str() {
                    "X" => (
                        PhysicsSource::X,
                        get_input_translation_x_from_normalized
                            as NormalizedPhysicsParameterValueGetter,
                    ),
                    "Y" => (
                        PhysicsSource::Y,
                        get_input_translation_y_from_normalized
                            as NormalizedPhysicsParameterValueGetter,
                    ),
                    "Angle" => (
                        PhysicsSource::Angle,
                        get_input_angle_from_normalized as NormalizedPhysicsParameterValueGetter,
                    ),
                    _ => {
                        panic!("Unknown physics input type: {}", input.kind);
                    }
                };
                let source = PhysicsParameter {
                    target_type: PhysicsTargetType::Parameter,
                    id: input.source.id,
                };
                let p_input = PhysicsInput::new(
                    source,
                    -1,
                    input.weight,
                    kind,
                    input.reflect,
                    get_norm_para_v,
                );
                inputs.push(p_input);
            }

            // Outputs
            for output in setting.output {
                let dest = PhysicsParameter {
                    target_type: PhysicsTargetType::Parameter,
                    id: output.destination.id,
                };
                let vertex_index = output.vertex_index;
                let weight = output.weight;
                let angle_scale = output.scale;
                let reflect = output.reflect;
                let (kind, get_value, get_scale) = match output.kind.as_str() {
                    "X" => (
                        PhysicsSource::X,
                        get_output_translation_x as PhysicsValueGetter,
                        get_output_scale_translation_x as PhysicsScaleGetter,
                    ),
                    "Y" => (
                        PhysicsSource::Y,
                        get_output_translation_y as PhysicsValueGetter,
                        get_output_scale_translation_y as PhysicsScaleGetter,
                    ),
                    "Angle" => (
                        PhysicsSource::Angle,
                        get_output_angle as PhysicsValueGetter,
                        get_output_scale_angle as PhysicsScaleGetter,
                    ),
                    _ => {
                        panic!("Unknown physics output type: {}", output.kind);
                    }
                };
                let p_output = PhysicsOutput::new(
                    dest,
                    -1,
                    vertex_index,
                    angle_scale,
                    weight,
                    kind,
                    reflect,
                    get_value,
                    get_scale,
                );
                outputs.push(p_output);
            }

            // Particles
            for v in setting.vertices {
                let pos = Vec2::new(v.position.x, v.position.y);

                let particle =
                    PhysicsParticle::new(v.mobility, v.delay, v.acceleration, v.radius, pos);

                particles.push(particle);
            }

            let sub_rig = PhysicsSubRig::new(
                inputs.len(),
                outputs.len(),
                particles.len(),
                0,
                0,
                0,
                norm_pos,
                norm_angle,
            );
            sub_rigs.push(sub_rig);
        }
        let rig = PhysicsRig::new(
            sub_rig_count,
            sub_rigs,
            inputs,
            outputs,
            particles,
            gravity,
            wind,
            fps,
        );
        Physics::new(rig)
    }

    pub fn evaluate(&mut self, model: &mut Model, delta_time: f32) {
        if delta_time <= 0. {
            return;
        }
        self.current_remain_time += delta_time;
        if self.current_remain_time > Self::MAX_DELTA_TIME {
            self.current_remain_time = 0.
        }

        let param_count = model.param_count;
        let (para_vs, para_max_vs, para_min_vs, para_default_vs) = unsafe {
            (
                slice::from_raw_parts_mut(csmGetParameterValues(model.model), param_count),
                slice::from_raw_parts(csmGetParameterMaximumValues(model.model), param_count),
                slice::from_raw_parts(csmGetParameterMinimumValues(model.model), param_count),
                slice::from_raw_parts(csmGetParameterDefaultValues(model.model), param_count),
            )
        };

        if self.parameter_caches.len() < param_count {
            self.parameter_caches.resize(param_count, 0.0);
        }
        if self.parameter_input_caches.len() < param_count {
            let cur_len = self.parameter_input_caches.len();
            self.parameter_input_caches.resize(param_count, 0.0);
            for j in cur_len..param_count {
                self.parameter_input_caches[j] = para_vs[j];
            }
        }

        if self.current_rig_outputs.is_empty() {
            for setting in &self.physics_rig.settings {
                self.current_rig_outputs
                    .push(vec![0.0; setting.output_count]);
                self.previous_rig_outputs
                    .push(vec![0.0; setting.output_count]);
            }
        }

        let p_delta_time = if self.physics_rig.fps > 0 {
            1.0 / (self.physics_rig.fps as f32)
        } else {
            delta_time
        };

        while self.current_remain_time >= p_delta_time {
            // Copy current_rig_outputs to previous_rig_outputs
            for setting_idx in 0..self.physics_rig.sub_rig_count {
                let out_count = self.physics_rig.settings[setting_idx].output_count;
                for i in 0..out_count {
                    self.previous_rig_outputs[setting_idx][i] = self.current_rig_outputs[setting_idx][i];
                }
            }

            let input_weight = p_delta_time / self.current_remain_time;
            for j in 0..param_count {
                self.parameter_caches[j] = self.parameter_input_caches[j] * (1.0 - input_weight) + para_vs[j] * input_weight;
                self.parameter_input_caches[j] = self.parameter_caches[j];
            }

            for setting_idx in 0..self.physics_rig.sub_rig_count {
                let setting = &self.physics_rig.settings[setting_idx];
                let mut total_angle = 0.0f32;
                let mut total_translation = Vec2::ZERO;

                let input_start = setting.base_input_index;
                let input_end = input_start + setting.input_count;
                for input in &mut self.physics_rig.inputs[input_start..input_end] {
                    let weight = input.weight / 100.0;

                    if input.source_parameter_index == -1 {
                        input.source_parameter_index = model.get_parameter_index(&input.source.id) as i32;
                    }
                    let src_idx = input.source_parameter_index as usize;

                    (input.get_normalized_parameter_value)(
                        &mut total_translation,
                        &mut total_angle,
                        self.parameter_caches[src_idx],
                        para_min_vs[src_idx],
                        para_max_vs[src_idx],
                        para_default_vs[src_idx],
                        &setting.normalization_position,
                        &setting.normalization_angle,
                        input.reflect,
                        weight,
                    );
                }

                let rad_angle = -total_angle.to_radians();
                let (sin_a, cos_a) = rad_angle.sin_cos();

                let rotated_translation = glam::Vec2::new(
                    total_translation.x * cos_a - total_translation.y * sin_a,
                    total_translation.x * sin_a + total_translation.y * cos_a,
                );

                let particle_start = setting.base_particle_index;
                let particle_end = particle_start + setting.particle_count;
                let particles_slice = &mut self.physics_rig.particles[particle_start..particle_end];

                let threshold = Self::MOVEMENT_THRESHOLD * setting.normalization_position.maximum;

                Self::update_particles(
                    particles_slice,
                    rotated_translation,
                    total_angle,
                    self.options.wind,
                    threshold,
                    p_delta_time,
                    Self::AIR_RESISTANCE,
                );

                let output_start = setting.base_output_index;
                let output_end = output_start + setting.output_count;
                for (i, output) in self.physics_rig.outputs[output_start..output_end]
                    .iter_mut()
                    .enumerate()
                {
                    let p_idx = output.vertex_index as usize;

                    if output.destination_parameter_index == -1 {
                        output.destination_parameter_index = model.get_parameter_index(&output.destination.id) as i32;
                    }

                    if p_idx < 1 || p_idx >= setting.particle_count {
                        continue;
                    }

                    let translation = particles_slice[p_idx].position - particles_slice[p_idx - 1].position;

                    let output_value = (output.get_value)(
                        translation,
                        particles_slice,
                        p_idx,
                        output.reflect,
                        self.options.gravity,
                    );

                    self.current_rig_outputs[setting_idx][i] = output_value;

                    let dest_idx = output.destination_parameter_index as usize;

                    Self::update_output_parameter_value(
                        &mut self.parameter_caches[dest_idx],
                        para_min_vs[dest_idx],
                        para_max_vs[dest_idx],
                        output_value,
                        output,
                    );
                }
            }

            self.current_remain_time -= p_delta_time;
        }

        let alpha = self.current_remain_time / p_delta_time;
        self.interpolate(model, alpha);
    }

    pub fn interpolate(&mut self, model: &mut Model, weight: f32) {
        let param_count = model.param_count;
        let (para_vs, para_max_vs, para_min_vs) = unsafe {
            (
                std::slice::from_raw_parts_mut(csmGetParameterValues(model.model), param_count),
                std::slice::from_raw_parts(csmGetParameterMaximumValues(model.model), param_count),
                std::slice::from_raw_parts(csmGetParameterMinimumValues(model.model), param_count),
            )
        };

        for (setting_idx, setting) in self.physics_rig.settings.iter().enumerate() {
            let start = setting.base_output_index;
            let end = start + setting.output_count;

            for (local_idx, output) in self.physics_rig.outputs[start..end].iter_mut().enumerate() {
                let dest_idx = output.destination_parameter_index;
                if dest_idx == -1 {
                    continue;
                }

                let dest_idx = dest_idx as usize;

                let interpolated_value = self.previous_rig_outputs[setting_idx][local_idx] * (1.0 - weight) + self.current_rig_outputs[setting_idx][local_idx] * weight;

                Self::update_output_parameter_value(
                    &mut para_vs[dest_idx],
                    para_min_vs[dest_idx],
                    para_max_vs[dest_idx],
                    interpolated_value,
                    output,
                );
            }
        }
    }

    pub fn update_output_parameter_value(
        para_v: &mut f32,
        para_v_min: f32,
        para_v_max: f32,
        translation: f32,
        output: &mut PhysicsOutput,
    ) {
        let output_scale = (output.get_scale)(output.translation_scale, output.angle_scale);
        let mut value = translation * output_scale;
        if value < para_v_min {
            if value < output.value_below_minimum {
                output.value_below_minimum = value;
            }
            value = para_v_min
        } else if value > para_v_max {
            if value > output.value_exceeded_maximum {
                output.value_exceeded_maximum = value
            }
            value = para_v_max;
        }

        let weight = output.weight / Self::MAX_WEIGHT;
        *para_v = if weight >= 1. {
            value
        } else {
            *para_v*(1. - weight) + value*weight
        }

    }

    pub fn update_particles(
        strand: &mut [PhysicsParticle],
        total_translation: Vec2,
        total_angel: f32,
        wind_direction: Vec2,
        threshold_value: f32,
        delta_time: f32,
        air_resistance: f32,
    ) {
        strand[0].position = total_translation;
        let total_radian = total_angel.to_radians();
        let current_gravity: Vec2 = total_radian.sin_cos().into();
        let current_gravity = current_gravity.normalize();
        for i in 0..strand.len() {
            strand[i].force = current_gravity * strand[i].acceleration + wind_direction;
            strand[i].last_position = strand[i].position;
            let delay = strand[i].delay * delta_time * 30.;
            let mut direction = strand[i].position - strand[i-1].position;
            let radian = direction_to_radian(strand[i].last_gravity, current_gravity) / air_resistance; 

            let rotation = Vec2::from_angle(radian);
            direction = Vec2::new(
                direction.x * rotation.x - direction.y * rotation.y,
                direction.x * rotation.y + direction.y * rotation.x,
            );
            strand[i].position = strand[i-1].position + direction;
            let velocity = Vec2::new(
                strand[i].velocity.x * delay,
                strand[i].velocity.y * delay
            );
            let force = strand[i].force * delay * delay;
            strand[i].position = strand[i].position + velocity + force;
            let mut new_direction = strand[i].position - strand[i - 1].position;
            new_direction = new_direction.normalize();
            strand[i].position = strand[i-1].position + new_direction * strand[i].radius;
            if strand[i].position.x.abs() < threshold_value {
                strand[i].position.x = 0.;
            }
            if delay != 0. {
                strand[i].velocity = (strand[i].position - strand[i].last_position) / delay * strand[i].mobility;
            }
            strand[i].force = Vec2::ZERO;
            strand[i].last_gravity = current_gravity;
        }
    }
}

pub fn normalize_parameter_value(
    value: f32,
    para_minimum: f32,
    para_maximum: f32,
    para_default: f32,
    norm_minimum: f32,
    norm_maximum: f32,
    norm_default: f32,
    is_inverted: bool,
) -> f32 {
    let mut result = 0.0;
    let clamped_value = value.clamp(para_minimum, para_maximum);

    if clamped_value < para_default {
        let range = para_default - para_minimum;
        if range != 0.0 {
            result = (clamped_value - para_default) / range;
        }
    } else {
        let range = para_maximum - para_default;
        if range != 0.0 {
            result = (clamped_value - para_default) / range;
        }
    }

    let sign = if is_inverted { -1.0 } else { 1.0 };
    let final_result = result * sign;

    if final_result < 0.0 {
        final_result * (norm_default - norm_minimum) + norm_default
    } else {
        final_result * (norm_maximum - norm_default) + norm_default
    }
}

pub fn get_input_translation_x_from_normalized(
    target_translation: &mut Vec2,
    _target_angle: &mut f32,
    value: f32,
    para_min_value: f32,
    para_max_value: f32,
    para_default_value: f32,
    norm_position: &PhysicsNormalization,
    _norm_angle: &PhysicsNormalization,
    is_inverted: bool,
    weight: f32,
) {
    target_translation.x += normalize_parameter_value(
        value,
        para_min_value,
        para_max_value,
        para_default_value,
        norm_position.minimum,
        norm_position.maximum,
        norm_position.default,
        is_inverted,
    ) * weight;
}

pub fn get_input_translation_y_from_normalized(
    target_translation: &mut Vec2,
    _target_angle: &mut f32,
    value: f32,
    para_min_value: f32,
    para_max_value: f32,
    para_default_value: f32,
    norm_position: &PhysicsNormalization,
    _norm_angle: &PhysicsNormalization,
    is_inverted: bool,
    weight: f32,
) {
    target_translation.y += normalize_parameter_value(
        value,
        para_min_value,
        para_max_value,
        para_default_value,
        norm_position.minimum,
        norm_position.maximum,
        norm_position.default,
        is_inverted,
    ) * weight;
}

pub fn get_input_angle_from_normalized(
    _target_translation: &mut Vec2,
    target_angle: &mut f32,
    value: f32,
    para_min_value: f32,
    para_max_value: f32,
    para_default_value: f32,
    _norm_position: &PhysicsNormalization,
    norm_angle: &PhysicsNormalization,
    is_inverted: bool,
    weight: f32,
) {
    *target_angle += normalize_parameter_value(
        value,
        para_min_value,
        para_max_value,
        para_default_value,
        norm_angle.minimum,
        norm_angle.maximum,
        norm_angle.default,
        is_inverted,
    ) * weight;
}

pub fn get_output_translation_x(
    translation: Vec2,
    _particles: &[PhysicsParticle],
    _particle_index: usize,
    is_inverted: bool,
    _parent_gravity: Vec2,
) -> f32 {
    let mut output_value = translation.x;
    if is_inverted {
        output_value *= -1.0;
    }
    output_value
}

pub fn get_output_translation_y(
    translation: Vec2,
    _particles: &[PhysicsParticle],
    _particle_index: usize,
    is_inverted: bool,
    _parent_gravity: Vec2,
) -> f32 {
    let mut output_value = translation.y;
    if is_inverted {
        output_value *= -1.0;
    }
    output_value
}

pub fn get_output_angle(
    translation: Vec2,
    particles: &[PhysicsParticle],
    particle_index: usize,
    is_inverted: bool,
    mut parent_gravity: Vec2,
) -> f32 {
    let mut output_value: f32;

    if particle_index >= 2 {
        parent_gravity =
            particles[particle_index - 1].position - particles[particle_index - 2].position;
    } else {
        parent_gravity *= -1.0;
    }

    output_value = direction_to_radian(parent_gravity, translation);

    if is_inverted {
        output_value *= -1.0;
    }
    output_value
}

pub fn get_output_scale_translation_x(translation_scale: Vec2, _angle_scale: f32) -> f32 {
    translation_scale.x
}

pub fn get_output_scale_translation_y(translation_scale: Vec2, _angle_scale: f32) -> f32 {
    translation_scale.y
}

pub fn get_output_scale_angle(_translation_scale: Vec2, angle_scale: f32) -> f32 {
    angle_scale
}

// math 

pub fn direction_to_radian(from: Vec2, to: Vec2) -> f32 {
    let q1 = to.y.atan2(to.x);
    let q2 = from.y.atan2(from.x);

    let mut ret = q1 - q2;

    while ret < -PI {
        ret += PI * 2.0;
    }
    while ret > PI {
        ret -= PI * 2.0;
    }

    ret
}

pub fn direction_to_degrees(from: Vec2, to: Vec2) -> f32 {
    let radian = direction_to_radian(from, to);
    let mut degree = radian.to_degrees();

    if (to.x - from.x) > 0.0 {
        degree = -degree;
    }

    degree
}
