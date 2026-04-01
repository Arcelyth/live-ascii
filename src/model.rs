use std::collections::HashMap;
use std::ffi::CStr;

use crate::ffi::*;

pub struct Model {
    pub model: *mut CsmModel,
    pub model_opacity: f32,

    pub param_count: usize,
    pub param_ids: Vec<String>,
    pub param_values: *mut f32,
    pub param_max_vs: *const f32,
    pub param_min_vs: *const f32,
    pub param_default_vs: *const f32,

    pub part_count: usize,
    pub part_ids: Vec<String>,
    pub part_opacities: *mut f32,

    pub drawable_count: usize,
    pub drawable_ids: Vec<String>,

    pub not_exist_param_values: HashMap<usize, f32>,
    pub not_exist_part_opacities: HashMap<usize, f32>,
    pub not_exist_part_id: HashMap<String, usize>,
    pub not_exist_param_id: HashMap<String, usize>,
    pub saved_params: Vec<f32>,
}

impl Model {
    pub fn new(model: *mut CsmModel) -> Self {
        unsafe {
            assert!(!model.is_null(), "csmModel pointer cannot be null");

            let param_count = csmGetParameterCount(model) as usize;
            let param_ids_ptr = csmGetParameterIds(model);
            let mut param_ids = Vec::with_capacity(param_count);
            for i in 0..param_count {
                let id_ptr = *param_ids_ptr.add(i);
                let id_str = CStr::from_ptr(id_ptr).to_string_lossy().into_owned();
                param_ids.push(id_str);
            }

            let part_count = csmGetPartCount(model) as usize;
            let part_ids_ptr = csmGetPartIds(model);
            let mut part_ids = Vec::with_capacity(part_count);
            for i in 0..part_count {
                let id_ptr = *part_ids_ptr.add(i);
                let id_str = CStr::from_ptr(id_ptr).to_string_lossy().into_owned();
                part_ids.push(id_str);
            }

            let drawable_count = csmGetDrawableCount(model) as usize;
            let drawable_ids_ptr = csmGetDrawableIds(model);
            let mut drawable_ids = Vec::with_capacity(drawable_count);
            for i in 0..drawable_count {
                let id_ptr = *drawable_ids_ptr.add(i);
                let id_str = CStr::from_ptr(id_ptr).to_string_lossy().into_owned();
                drawable_ids.push(id_str);
            }

            Self {
                model,
                model_opacity: 1.,
                param_count,
                param_ids,
                param_values: csmGetParameterValues(model),
                param_max_vs: csmGetParameterMaximumValues(model),
                param_min_vs: csmGetParameterMinimumValues(model),
                param_default_vs: csmGetParameterDefaultValues(model),

                part_count,
                part_ids,
                part_opacities: csmGetPartOpacities(model),

                drawable_count,
                drawable_ids,

                not_exist_param_values: HashMap::new(),
                not_exist_part_opacities: HashMap::new(),
                not_exist_param_id: HashMap::new(),
                not_exist_part_id: HashMap::new(),
                saved_params: vec![],
            }
        }
    }

    pub fn update(&mut self) {
        unsafe {
            csmUpdateModel(self.model);
            csmResetDrawableDynamicFlags(self.model);
        }
    }

    pub fn get_parameter_index(&self, id: &str) -> Option<usize> {
        self.param_ids.iter().position(|p| p == id)
    }

    pub fn get_parameter_value(&self, idx: usize) -> f32 {
        unsafe {
            if let Some(v) = self.not_exist_param_values.get(&idx) {
                *v
            } else {
                *self.param_values.add(idx)
            }
        }
    }

    pub fn get_parameter_value_by_id(&self, idx: &str) -> Option<f32> {
        if let Some(index) = self.get_parameter_index(idx) {
            Some(self.get_parameter_value(index))
        } else {
            None
        }
    }

    pub fn set_parameter_value(&mut self, index: usize, value: f32, weight: f32) {
        if index >= self.param_count {
            let current_val = *self.not_exist_param_values.get(&index).unwrap_or(&0.0);

            let new_val = if weight == 1.0 {
                value
            } else {
                current_val * (1.0 - weight) + (value * weight)
            };

            self.not_exist_param_values.insert(index, new_val);
            return;
        }

        let mut processed_value = value;

        if self.is_repeat(index) {
            processed_value = self.get_parameter_repeat_value(index, processed_value);
        } else {
            unsafe {
                let min_val = *self.param_min_vs.add(index);
                let max_val = *self.param_max_vs.add(index);
                processed_value = processed_value.clamp(min_val, max_val);
            }
        }

        unsafe {
            let current_ptr = self.param_values.add(index);
            let current_val = *current_ptr;

            let final_val = if weight == 1.0 {
                processed_value
            } else {
                (current_val * (1.0 - weight)) + (processed_value * weight)
            };

            *current_ptr = final_val;
        }
    }


    pub fn set_parameter_value_by_id(&mut self, index: &str, value: f32, weight: f32) {
        if let Some(index) = self.get_parameter_index(index) {
            self.set_parameter_value(index, value, weight);
        };
    }

    pub fn is_repeat(&self, parameter_index: usize) -> bool {
        if self.not_exist_param_values.contains_key(&parameter_index) {
            return false;
        }

        assert!(parameter_index < self.param_count, "Index out of bounds");

        unsafe {
            let repeats_ptr = csmGetParameterRepeats(self.model);
            let is_repeat = *repeats_ptr.add(parameter_index);

            is_repeat != 0
        }
    }

    pub fn get_parameter_repeat_value(&self, parameter_index: usize, mut value: f32) -> f32 {
        if self.not_exist_param_values.contains_key(&parameter_index) {
            return value;
        }

        assert!(parameter_index < self.param_count);

        unsafe {
            let max_value = *self.param_max_vs.add(parameter_index);
            let min_value = *self.param_min_vs.add(parameter_index);
            let value_size = max_value - min_value;

            if value > max_value {
                let over_value = (value - max_value) % value_size;
                if !over_value.is_nan() {
                    value = min_value + over_value;
                } else {
                    value = max_value;
                }
            }

            if value < min_value {
                let over_value = (min_value - value) % value_size;
                if !over_value.is_nan() {
                    value = max_value - over_value;
                } else {
                    value = min_value;
                }
            }
        }

        value
    }

    pub unsafe fn get_canvas_info(&self) -> (CsmVector2, CsmVector2, f32) {
        let mut size_in_pixels = CsmVector2 { x: 0.0, y: 0.0 };
        let mut origin_in_pixels = CsmVector2 { x: 0.0, y: 0.0 };
        let mut pixels_per_unit = 0.0;

        unsafe {
            csmReadCanvasInfo(
                self.model,
                &mut size_in_pixels,
                &mut origin_in_pixels,
                &mut pixels_per_unit,
            );
        }

        (size_in_pixels, origin_in_pixels, pixels_per_unit)
    }

    pub fn get_render_orders(&self) -> &[i32] {
        unsafe {
            std::slice::from_raw_parts(csmGetDrawableRenderOrders(self.model), self.drawable_count)
        }
    }
}
