use std::error::Error;
use std::fs;

use crate::ffi::{csmGetParameterValues, csmGetPartOpacities};
use crate::motion::json::*;
use crate::renderer::*;

pub fn lerp_points(a: SegmentPoint, b: SegmentPoint, t: f32) -> SegmentPoint {
    SegmentPoint {
        time: a.time + (b.time - a.time) * t,
        value: a.value + (b.value - a.value) * t,
    }
}

pub fn linear_evaluate(p0: SegmentPoint, p1: SegmentPoint, time: f32) -> f32 {
    let mut t = (time - p0.time) / (p1.time - p0.time);
    if t < 0. {
        t = 0.;
    }
    p0.value + ((p1.value - p0.value) * t)
}

pub fn bezier_evaluate(
    p0: SegmentPoint,
    p1: SegmentPoint,
    p2: SegmentPoint,
    p3: SegmentPoint,
    time: f32,
) -> f32 {
    let mut t = (time - p0.time) / (p3.time - p0.time);
    if t < 0. {
        t = 0.;
    }

    let p01 = lerp_points(p0, p1, t);
    let p12 = lerp_points(p1, p2, t);
    let p23 = lerp_points(p2, p3, t);
    let p012 = lerp_points(p01, p12, t);
    let p123 = lerp_points(p12, p23, t);
    lerp_points(p012, p123, t).value
}

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
            if self.motion.meta.loop_ {
                self.current_time %= self.motion.meta.duration;
            } else {
                self.current_time = self.motion.meta.duration;
            }
        }

        for curve in &self.motion.curves {
            let val = self.evaluate_curve(&curve.segments, self.current_time);

            if curve.target == "Parameter" {
                unsafe {
                    if let Some(idx) = renderer.find_param_index(&curve.id) {
                        let param_values = csmGetParameterValues(renderer.model.model);
                        *param_values.add(idx) = val;
                    }
                }
            } else if curve.target == "PartOpacity" {
                unsafe {
                    if let Some(idx) = renderer.find_part_index(&curve.id) {
                        let part_opacities = csmGetPartOpacities(renderer.model.model);
                        *part_opacities.add(idx) = val;
                    }
                }
            }
        }
    }

    fn evaluate_curve(&self, segments: &Segments, time: f32) -> f32 {
        let segs = &segments.0;
        if segs.is_empty() {
            return 0.0;
        }

        for seg in segs {
            match seg {
                SegmentType::Linear(p0, p1) => {
                    if time <= p1.time {
                        if time <= p0.time {
                            return p0.value;
                        }
                        return linear_evaluate(*p0, *p1, time);
                    }
                }
                SegmentType::Bezier(p) => {
                    if time <= p[3].time {
                        if time <= p[0].time {
                            return p[0].value;
                        }
                        return bezier_evaluate(p[0], p[1], p[2], p[3], time);
                    }
                }
                SegmentType::Stepped(p0, p1) => {
                    if time < p1.time {
                        return p0.value;
                    }
                }
                SegmentType::InverseStepped(_, p1) => {
                    if time <= p1.time {
                        return p1.value;
                    }
                }
            }
        }

        match segs.last().unwrap() {
            SegmentType::Linear(_, p1) => p1.value,
            SegmentType::Bezier([.., p3]) => p3.value,
            SegmentType::Stepped(_, p1) => p1.value,
            SegmentType::InverseStepped(_, p1) => p1.value,
        }
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
        
        let expected_segments = Segments(vec![
            SegmentType::Linear(
                SegmentPoint { time: 0., value: 0. },
                SegmentPoint { time: 1., value: 30. },
            ),
            SegmentType::Linear(
                SegmentPoint { time: 1., value: 30. },
                SegmentPoint { time: 2., value: -30. },
            ),
            SegmentType::Linear(
                SegmentPoint { time: 2., value: -30. },
                SegmentPoint { time: 3., value: 30. },
            ),
            SegmentType::Linear(
                SegmentPoint { time: 3., value: 30. },
                SegmentPoint { time: 4., value: 0. },
            ),
        ]);

        let c1 = Curve {
            target: "Parameter".to_string(),
            id: "PARAM_ANGLE_X".to_string(),
            fade_in_time: 1.0,
            fade_out_time: 1.0,
            segments: expected_segments,
        };
        
        assert_eq!(mp.motion.curves[0], c1);
    }
}
