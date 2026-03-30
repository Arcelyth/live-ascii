use std::fmt;

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
    // TODO: change to Segments
    pub segments: Vec<f32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
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
pub struct Segments(Vec<SegmentType>);

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


