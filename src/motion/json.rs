use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Deserialize, Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MotionMeta {
    pub duration: f32,
    pub fps: f32,
    pub loop_: bool,
    #[serde(default)]
    pub are_beziers_restricted: bool,
    pub curve_count: i32,
    pub total_segment_count: i32,
    pub total_point_count: i32,
    #[serde(default)]
    pub fade_in_time: Option<f32>,
    #[serde(default)]
    pub fade_out_time: Option<f32>,
    #[serde(default)]
    pub user_data_count: i32,
    #[serde(default)]
    pub total_user_data_size: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Curve {
    pub target: String,
    pub id: String,
    #[serde(default = "crate::utils::default_fade_time")]
    pub fade_in_time: f32,
    #[serde(default = "crate::utils::default_fade_time")]
    pub fade_out_time: f32,
    pub segments: Segments,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct MotionEvent {
    pub time: f32,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Motion3 {
    pub version: usize,
    pub meta: MotionMeta,
    pub curves: Vec<Curve>,
    #[serde(default)]
    pub user_data: Vec<MotionEvent>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SegmentPoint {
    pub time: f32,
    pub value: f32,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SegmentType {
    Linear(SegmentPoint, SegmentPoint),
    Bezier([SegmentPoint; 4]),
    Stepped(SegmentPoint, SegmentPoint),
    InverseStepped(SegmentPoint, SegmentPoint),
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Segments(pub Vec<SegmentType>);

impl<'de> Deserialize<'de> for Segments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SegVisitor;

        impl<'de> Visitor<'de> for SegVisitor {
            type Value = Segments;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a sequence of numbers")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut result = vec![];
                let time: f32 = seq.next_element()?.unwrap();
                let value: f32 = seq.next_element()?.unwrap();
                let mut last_point = SegmentPoint { time, value };

                while let Some(seg_ty) = seq.next_element()? as Option<i32> {
                    match seg_ty {
                        0 => {
                            // Linear
                            let time: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing linear time"))?;
                            let value: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing linear value"))?;
                            let p = SegmentPoint { time, value };
                            result.push(SegmentType::Linear(last_point, p));
                            last_point = p;
                        }
                        1 => {
                            // Bezier
                            let t0: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing bezier t0"))?;
                            let v0: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing bezier v0"))?;
                            let t1: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing bezier t1"))?;
                            let v1: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing bezier v1"))?;
                            let t2: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing bezier t2"))?;
                            let v2: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing bezier v2"))?;
                            let p = SegmentPoint {
                                time: t2,
                                value: v2,
                            };
                            result.push(SegmentType::Bezier([
                                last_point,
                                SegmentPoint {
                                    time: t0,
                                    value: v0,
                                },
                                SegmentPoint {
                                    time: t1,
                                    value: v1,
                                },
                                p,
                            ]));
                            last_point = p;
                        }
                        2 => {
                            // Stepped
                            let t: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing stepped time"))?;
                            let v: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing stepped value"))?;
                            let p = SegmentPoint { time: t, value: v };
                            result.push(SegmentType::Stepped(last_point, p));
                            last_point = p;
                        }
                        3 => {
                            // Inverse Stepped
                            let t: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing inv-stepped time"))?;
                            let v: f32 = seq
                                .next_element()?
                                .ok_or_else(|| de::Error::custom("Missing inv-stepped value"))?;
                            let p = SegmentPoint { time: t, value: v };
                            result.push(SegmentType::InverseStepped(last_point, p));
                            last_point = p;
                        }
                        _ => return Err(de::Error::custom("Unknown segment format.")),
                    }
                }
                Ok(Segments(result))
            }
        }
        deserializer.deserialize_seq(SegVisitor)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CurveTargetType {
    Parameter,
    PartOpacity,
    Model,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionCurve {
    pub target_type: CurveTargetType,
    pub id: String,
    pub segment_count: usize,
    pub base_segment_index: usize,
    pub fade_in_time: f32,
    pub fade_out_time: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MotionSegmentType {
    Linear,
    Bezier,
    Stepped,
    InverseStepped,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionSegment {
    pub base_point_index: usize,
    pub segment_type: MotionSegmentType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MotionData {
    pub duration: f32,
    pub loop_: bool,
    pub fps: f32,
    pub curves: Vec<MotionCurve>,
    pub segments: Vec<MotionSegment>,
    pub points: Vec<SegmentPoint>,
    pub events: Vec<MotionEvent>,
}

impl MotionData {
    pub fn from_path(base_dir: &str, path: &str) -> Result<Self, Box<dyn Error>> {
        let full_path = Path::new(base_dir).join(path);
        let data = fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read file {:?}: {}", full_path, e))?;

        let m3: Motion3 = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse JSON ({:?}): {}", full_path, e))?;
        let mut curves = Vec::with_capacity(m3.curves.len());
        let mut segments = Vec::new();
        let mut points = Vec::new();

        for curve in m3.curves {
            let base_segment_index = segments.len();
            let mut segment_count = 0;

            for seg in curve.segments.0 {
                let base_point_index = points.len();

                match seg {
                    SegmentType::Linear(p0, p1) => {
                        if segment_count == 0 {
                            points.push(p0);
                        }
                        points.push(p1);
                        segments.push(MotionSegment {
                            base_point_index,
                            segment_type: MotionSegmentType::Linear,
                        });
                    }
                    SegmentType::Bezier(p_arr) => {
                        if segment_count == 0 {
                            points.push(p_arr[0]);
                        }
                        points.push(p_arr[1]);
                        points.push(p_arr[2]);
                        points.push(p_arr[3]);
                        segments.push(MotionSegment {
                            base_point_index,
                            segment_type: MotionSegmentType::Bezier,
                        });
                    }
                    SegmentType::Stepped(p0, p1) => {
                        if segment_count == 0 {
                            points.push(p0);
                        }
                        points.push(p1);
                        segments.push(MotionSegment {
                            base_point_index,
                            segment_type: MotionSegmentType::Stepped,
                        });
                    }
                    SegmentType::InverseStepped(p0, p1) => {
                        if segment_count == 0 {
                            points.push(p0);
                        }
                        points.push(p1);
                        segments.push(MotionSegment {
                            base_point_index,
                            segment_type: MotionSegmentType::InverseStepped,
                        });
                    }
                }
                segment_count += 1;
            }

            let target_type = match curve.target.as_str() {
                "Parameter" => CurveTargetType::Parameter,
                "PartOpacity" => CurveTargetType::PartOpacity,
                "Model" => CurveTargetType::Model,
                _ => panic!("Unknow target type."),
            };

            curves.push(MotionCurve {
                target_type,
                id: curve.id,
                segment_count,
                base_segment_index,
                fade_in_time: curve.fade_in_time,
                fade_out_time: curve.fade_out_time,
            });
        }

        Ok(Self {
            duration: m3.meta.duration,
            loop_: m3.meta.loop_,
            fps: m3.meta.fps,
            curves,
            segments,
            points,
            events: m3.user_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_motion3_full_structure_comparison() {
        let result = MotionData::from_path("./test_file", "test.motion3.json")
            .expect("Failed to parse test.motion3.json");

        let expected = MotionData {
            duration: 2.0,
            loop_: true,
            fps: 30.0,
            curves: vec![
                MotionCurve {
                    target_type: CurveTargetType::Parameter,
                    id: "PARAM_ANGLE_X".into(),
                    segment_count: 2,
                    base_segment_index: 0,
                    fade_in_time: -1.0,
                    fade_out_time: -1.0,
                },
                MotionCurve {
                    target_type: CurveTargetType::Parameter,
                    id: "ParamArmRA".into(),
                    segment_count: 2,
                    base_segment_index: 2,
                    fade_in_time: -1.0,
                    fade_out_time: -1.0,
                },
            ],
            segments: vec![
                MotionSegment {
                    base_point_index: 0,
                    segment_type: MotionSegmentType::Linear,
                },
                MotionSegment {
                    base_point_index: 2,
                    segment_type: MotionSegmentType::Linear,
                },
                MotionSegment {
                    base_point_index: 3,
                    segment_type: MotionSegmentType::Bezier,
                },
                MotionSegment {
                    base_point_index: 7,
                    segment_type: MotionSegmentType::Linear,
                },
            ],
            points: vec![
                // PARAM_ANGLE_X
                SegmentPoint {
                    time: 0.0,
                    value: 0.0,
                }, // 0
                SegmentPoint {
                    time: 1.0,
                    value: 30.0,
                }, // 1
                SegmentPoint {
                    time: 2.0,
                    value: 0.0,
                }, // 2
                // ParamArmRA
                SegmentPoint {
                    time: 0.0,
                    value: 0.0,
                }, // 3
                SegmentPoint {
                    time: 0.3,
                    value: 0.0,
                }, // 4
                SegmentPoint {
                    time: 0.7,
                    value: 10.0,
                }, // 5
                SegmentPoint {
                    time: 1.0,
                    value: 10.0,
                }, // 6
                SegmentPoint {
                    time: 2.0,
                    value: 0.0,
                }, // 7
            ],
            events: vec![],
        };

        assert_eq!(
            result, expected,
            "The parsed MotionData does not match the expected structure."
        );
    }
}
