use std::error::Error;
use std::fs;

use crate::ffi::{csmGetParameterValues, csmGetPartOpacities};
use crate::renderer::*;
use crate::motion::json::*;


pub struct MotionPlayer {
    pub motion: Motion3,
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
            let val = self.evaluate_curve(&curve.segments, self.current_time);
            if curve.target == "Parameter" {
                unsafe {
                    if let Some(idx) = renderer.find_param_index(&curve.id) {
                        let param_values = csmGetParameterValues(renderer.model);
                        *param_values.add(idx) = val;
                    }
                }
            } else if curve.target == "PartOpacity" {
                unsafe {
                    if let Some(idx) = renderer.find_part_index(&curve.id) {
                        let part_opacities = csmGetPartOpacities(renderer.model);
                        *part_opacities.add(idx) = val;
                    }
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
    fn parse_motion3_json() {
        let mp = MotionPlayer::new("./test_file/test.motion3.json").unwrap();
        assert_eq!(mp.motion.version, 3);
        assert_eq!(mp.motion.meta.duration, 4.);
        assert_eq!(mp.motion.meta.fps, 30.);
        assert_eq!(mp.motion.meta.loop_, true);
        let c1 = Curve {
            target: "Parameter".to_string(),
            id: "PARAM_ANGLE_X".to_string(),
            fade_in_time: 1.0,
            fade_out_time: 1.0,
            segments: vec![0., 0., 0., 1., 30., 0., 2., -30., 0., 3., 30., 0., 4., 0.],
        };
        assert_eq!(mp.motion.curves[0], c1);
    }
}
