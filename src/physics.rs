use glam::Vec2;

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

pub type PhysicsScaleGetter = fn(
    translation_scale: Vec2,
    angle_scale: f32,
) -> f32;


#[derive(Clone)]
pub struct PhysicsInput {
    pub source: PhysicsParameter,
    pub source_parameter_index: usize,
    pub weight: f32,
    pub kind: PhysicsSource, // type
    pub reflect: bool,
    pub get_normalized_parameter_value: NormalizedPhysicsParameterValueGetter,
}

#[derive(Clone)]
pub struct PhysicsOutput {
    pub destination: PhysicsParameter,
    pub destination_parameter_index: usize,
    pub vertex_index: usize,
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

#[derive(Debug, Clone)]
pub struct Options {
    pub gravity: Vec2,
    pub wind: Vec2,
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
    pub fps: f32,
}

#[derive(Clone)]
pub struct Physics {
    pub options: Options,
    pub physics_rig: Vec<PhysicsRig>,
    pub current_rig_outputs: Vec<Vec<f32>>,
    pub previous_rig_outputs: Vec<Vec<f32>>,
    pub current_remain_time: f32,
    pub parameter_caches: Vec<f32>,
    pub parameter_input_caches: Vec<f32>,
}
