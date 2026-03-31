use std::collections::HashMap;
use std::ffi::CStr;

use crate::ffi::*;

pub struct Model {
    pub model: *mut CsmModel,

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

    pub not_exist_param_values: HashMap<String, f32>,
    pub not_exist_part_opacities: HashMap<String, f32>,
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

    pub fn get_parameter_value(&self, id: &str) -> f32 {
        unsafe {
            if let Some(index) = self.get_parameter_index(id) {
                *self.param_values.add(index)
            } else {
                *self.not_exist_param_values.get(id).unwrap_or(&0.0)
            }
        }
    }

    pub fn set_parameter_value(&mut self, id: &str, value: f32, weight: f32) {
        if let Some(index) = self.get_parameter_index(id) {
            unsafe {
                let min_val = *self.param_min_vs.add(index);
                let max_val = *self.param_max_vs.add(index);
                let clamped_value = value.clamp(min_val, max_val);

                let current_val = *self.param_values.add(index);
                let new_val = if weight == 1.0 {
                    clamped_value
                } else {
                    current_val * (1.0 - weight) + clamped_value * weight
                };
                *self.param_values.add(index) = new_val;
            }
        } else {
            let current_val = *self.not_exist_param_values.get(id).unwrap_or(&0.0);
            let new_val = if weight == 1.0 {
                value
            } else {
                current_val * (1.0 - weight) + value * weight
            };
            self.not_exist_param_values.insert(id.to_string(), new_val);
        }
    }

    pub unsafe fn add_parameter_value(&mut self, id: &str, value: f32, weight: f32) {
        let current_val = self.get_parameter_value(id);
        self.set_parameter_value(id, current_val + (value * weight), 1.0);
    }

    pub unsafe fn multiply_parameter_value(&mut self, id: &str, value: f32, weight: f32) {
        let current_val = self.get_parameter_value(id);
        self.set_parameter_value(id, current_val * (1.0 + (value - 1.0) * weight), 1.0);
    }

    pub fn get_part_index(&self, id: &str) -> Option<usize> {
        self.part_ids.iter().position(|p| p == id)
    }

    pub fn set_part_opacity(&mut self, id: &str, opacity: f32) {
        if let Some(index) = self.get_part_index(id) {
            unsafe {
                *self.part_opacities.add(index) = opacity;
            }
        } else {
            self.not_exist_part_opacities
                .insert(id.to_string(), opacity);
        }
    }

    pub fn get_part_opacity(&self, id: &str) -> f32 {
        if let Some(index) = self.get_part_index(id) {
            unsafe { *self.part_opacities.add(index) }
        } else {
            *self.not_exist_part_opacities.get(id).unwrap_or(&0.0)
        }
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
        unsafe { std::slice::from_raw_parts(csmGetDrawableRenderOrders(self.model), self.drawable_count) }
    }
}

