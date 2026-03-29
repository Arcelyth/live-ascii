use std::error::Error;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::renderer::*;
use crate::ffi::csmGetParameterValues;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Meta {
    pub duration: f32,
    pub fps: f32,
    pub loop_: bool,
    // not necessary
//  pub are_beziers_restricted: bool,
//  pub curve_count: usize,
//  pub total_segment_count: usize,
//  pub total_point_count: usize,
//  pub user_data_count: usize,
//  pub total_user_data_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Curve {
    pub target: String,
    pub id: String,
    pub segments: Vec<f32>,
}

impl PartialEq for Curve {
    fn eq(&self, other: &Self) -> bool {
        (self.target == other.target) && (self.id == other.id) && (self.segments == other.segments)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Motion {
    pub version: usize,
    pub meta: Meta,
    pub curves: Vec<Curve>,
}

pub struct MotionPlayer {
    pub motion: Motion,
    pub current_time: f32,
}

impl MotionPlayer {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        let motion = serde_json::from_str(&data)?;
        Ok(Self {
            motion,
            current_time: 0.,
        })
    }

    pub fn update(&mut self, delta_time: f32, renderer: &mut Renderer) {
        self.current_time += delta_time;

        // loop motion
        if self.current_time >= self.motion.meta.duration {
            self.current_time = 0.
        }

        for curve in &self.motion.curves {
            if curve.target != "Parameter" {
                continue;
            }

            let val = self.evaluate_curve(&curve.segments, self.current_time);
            unsafe {
                if let Some(idx) = renderer.find_param_index(&curve.id) {
                    let param_values = csmGetParameterValues(renderer.model);
                    *param_values.add(idx) = val;
                }
            }
        }
    }

    fn evaluate_curve(&self, segments: &[f32], time: f32) -> f32 {
        let mut i = 0;
        let mut base_time = segments[0];
        let mut base_value = segments[1];

        if time <= base_time {
            return base_value;
        }

        i += 2;

        while i < segments.len() {
            let segment_type = segments[i] as i32;
            i += 1;

            match segment_type {
                0 => {
                    // Linear
                    let next_time = segments[i];
                    let next_value = segments[i + 1];
                    i += 2;

                    if time <= next_time {
                        let t = (time - base_time) / (next_time - base_time);
                        return base_value + (next_value - base_value) * t;
                    }
                    base_time = next_time;
                    base_value = next_value;
                }
                1 => {
                    // TODO: Bezier
                    let next_time = segments[i + 4];
                    let next_value = segments[i + 5];
                    i += 6;

                    if time <= next_time {
                        let t = (time - base_time) / (next_time - base_time);
                        return base_value + (next_value - base_value) * t;
                    }
                    base_time = next_time;
                    base_value = next_value;
                }
                2 => {
                    // Stepped
                    let next_time = segments[i];
                    let next_value = segments[i + 1];
                    i += 2;

                    if time < next_time {
                        return base_value;
                    }
                    base_time = next_time;
                    base_value = next_value;
                }
                _ => break, // Inverse Stepped
            }
        }
        base_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_motion_json() {
        let mp = MotionPlayer::new("./test_file/test.motion3.json").unwrap();
        assert_eq!(mp.motion.version, 3);
        assert_eq!(mp.motion.meta.duration, 4.);
        assert_eq!(mp.motion.meta.fps, 30.);
        assert_eq!(mp.motion.meta.loop_, true);
        let c1 = Curve {
            target: "Parameter".to_string(),
            id: "PARAM_ANGLE_X".to_string(),
            segments: vec![0., 0., 0., 1., 30., 0., 2., -30., 0., 3., 30., 0., 4., 0.],
        };
        assert_eq!(mp.motion.curves[0], c1);
    } 
}
