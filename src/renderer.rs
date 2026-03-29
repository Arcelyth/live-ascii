#![allow(dead_code)]

use std::error::Error;
use std::ffi::CStr;
use std::io::{Write, stdout};
use std::time::Duration;
use std::time::Instant;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self},
};
use image::{DynamicImage, GenericImageView};

use crate::context::*;
use crate::ffi::*;
use crate::geometry::*;
use crate::motion::*;

const ASCII_CHARS: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

pub struct Renderer {
    pub count: usize,
    pub model: *mut CsmModel,
    constant_flags: *const u8,
    texture_indices: *const i32,
    vertex_counts: *const i32,
    vertex_positions: *const *const CsmVector2,
    vertex_uvs: *const *const CsmVector2,
    index_counts: *const i32,
    indices: *const *const u16,
    multiply_colors: *const CsmVector4,
    screen_colors: *const CsmVector4,
    textures: Vec<DynamicImage>,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    start_time: Instant,
}

impl Renderer {
    pub fn new(model_ptr: *mut CsmModel, textures: Vec<DynamicImage>) -> Self {
        unsafe {
            Self {
                model: model_ptr,
                count: csmGetDrawableCount(model_ptr) as usize,
                constant_flags: csmGetDrawableConstantFlags(model_ptr),
                texture_indices: csmGetDrawableTextureIndices(model_ptr),
                vertex_counts: csmGetDrawableVertexCounts(model_ptr),
                vertex_positions: csmGetDrawableVertexPositions(model_ptr),
                vertex_uvs: csmGetDrawableVertexUvs(model_ptr),
                index_counts: csmGetDrawableIndexCounts(model_ptr),
                indices: csmGetDrawableIndices(model_ptr),
                multiply_colors: csmGetDrawableMultiplyColors(model_ptr),
                screen_colors: csmGetDrawableScreenColors(model_ptr),
                textures,
                offset_x: 0.,
                offset_y: 0.,
                scale: 1.,
                start_time: Instant::now(),
            }
        }
    }

    pub fn render(
        &mut self,
        context: &mut Context,
        mp: &mut Option<MotionPlayer>,
    ) -> Result<(), Box<dyn Error>> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), cursor::Hide)?;
        let fps = 120.0;
        let target_frame_time = Duration::from_secs_f64(1.0 / fps);
        let mut last_frame = Instant::now();
        loop {
            let frame_start = Instant::now();

            if event::poll(Duration::from_millis(1))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => self.offset_y -= 0.1,
                        KeyCode::Down => self.offset_y += 0.1,
                        KeyCode::Left => self.offset_x -= 0.1,
                        KeyCode::Right => self.offset_x += 0.1,
                        KeyCode::Char('=') | KeyCode::Char('+') => self.scale *= 1.1,
                        KeyCode::Char('-') => self.scale *= 0.9,
                        _ => {}
                    }
                }
            }
            context.update()?;
            context.clear();

            let delta_time = last_frame.elapsed().as_secs_f32();
            last_frame = Instant::now();
            if let Some(mp) = mp {
                mp.update(delta_time, self);
            }

            // manipulation of model
            let t = self.start_time.elapsed().as_secs_f32();
            unsafe {
                // updating parameter and parts opacity
                let p_count = csmGetParameterCount(self.model);
                let p_ids = csmGetParameterIds(self.model);
                let p_min_vs = csmGetParameterMinimumValues(self.model);
                let p_max_vs = csmGetParameterMaximumValues(self.model);
                let p_default_vs = csmGetParameterDefaultValues(self.model);
                let p_vs = csmGetParameterValues(self.model);

                let part_count = csmGetPartCount(self.model);
                let part_ids = csmGetPartIds(self.model);
                let part_opacities = csmGetPartOpacities(self.model);

                //                for i in 0..p_count {
                //                    let max_v = *p_max_vs.add(i as usize);
                //                    let min_v = *p_min_vs.add(i as usize);
                //                    let range = max_v - min_v;
                //                    let mid = min_v + range / 2.0;
                //
                //                    let phase = i as f32 * 0.3;
                //
                //                    let current_val = mid + (range / 2.0) * (t * 2.0 + phase).sin();
                //
                //                    *p_vs.add(i as usize) = current_val;
                //                }
            }

            // applying manioulation to Drawable
            unsafe {
                csmResetDrawableDynamicFlags(self.model);
                // updating vertex information
                csmUpdateModel(self.model);
            }

            // applying updated vertex inforamation to renderer
            let (dy_flags, opacities, vt_positions, render_orders) = unsafe {
                let dy_flags = csmGetDrawableDynamicFlags(self.model);
                let opacities = csmGetDrawableOpacities(self.model);
                let vt_positions = csmGetDrawableVertexPositions(self.model);
                let render_orders = csmGetDrawableRenderOrders(self.model);
                (dy_flags, opacities, vt_positions, render_orders)
            };

            // rendering a model
            let mask_counts = unsafe {
                // Clipping
                let mask_counts = csmGetDrawableMaskCounts(self.model);

                // Multiply color, Screen color
                self.multiply_colors = csmGetDrawableMultiplyColors(self.model);
                self.screen_colors = csmGetDrawableScreenColors(self.model);
                mask_counts
            };

            // sort by render_orders in ascending order
            let mut drawables: Vec<usize> = (0..self.count).collect();
            drawables.sort_by_key(|&i| unsafe { *render_orders.add(i) });

            for &drawable_idx in &drawables {
                unsafe {
                    let opacity = *opacities.add(drawable_idx);
                    if opacity <= 0.0 {
                        continue;
                    }
                    // get texture
                    let tex_idx = *self.texture_indices.add(drawable_idx) as usize;
                    if tex_idx >= self.textures.len() {
                        continue;
                    }
                    let current_texture = &self.textures[tex_idx];
                    let img_w = current_texture.width();
                    let img_h = current_texture.height();

                    let index_count = *self.index_counts.add(drawable_idx) as usize;
                    let indices_ptr = *self.indices.add(drawable_idx);
                    let vertices_ptr = *vt_positions.add(drawable_idx);
                    let uvs_ptr = *self.vertex_uvs.add(drawable_idx);

                    for i in (0..index_count).step_by(3) {
                        let i0 = *indices_ptr.add(i) as usize;
                        let i1 = *indices_ptr.add(i + 1) as usize;
                        let i2 = *indices_ptr.add(i + 2) as usize;

                        let v0 = self.transform_to_screen(
                            *vertices_ptr.add(i0),
                            context.width,
                            context.height,
                        );
                        let v1 = self.transform_to_screen(
                            *vertices_ptr.add(i1),
                            context.width,
                            context.height,
                        );
                        let v2 = self.transform_to_screen(
                            *vertices_ptr.add(i2),
                            context.width,
                            context.height,
                        );

                        let triangle = Triangle::new(v0, v1, v2);

                        // get bounding box
                        let bbox = triangle.get_box();
                        let min_x = bbox.minx.max(0.0) as u16;
                        let max_x = bbox.maxx.min((context.width - 1) as f32) as u16;
                        let min_y = bbox.miny.max(0.0) as u16;
                        let max_y = bbox.maxy.min((context.height - 1) as f32) as u16;

                        let total_area = triangle.signed_area();

                        for y in min_y..=max_y {
                            for x in min_x..=max_x {
                                let p = Vec3 {
                                    x: x as f32,
                                    y: y as f32,
                                    z: 0.0,
                                };

                                let w0 = Triangle::new(v1, v2, p).signed_area() / total_area;
                                let w1 = Triangle::new(v2, v0, p).signed_area() / total_area;
                                let w2 = 1. - w0 - w1;

                                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                                    let uv0 = *uvs_ptr.add(i0);
                                    let uv1 = *uvs_ptr.add(i1);
                                    let uv2 = *uvs_ptr.add(i2);
                                    let interp_u = w0 * uv0.x + w1 * uv1.x + w2 * uv2.x;
                                    let interp_v = w0 * uv0.y + w1 * uv1.y + w2 * uv2.y;
                                    let u = interp_u.clamp(0.0, 1.0);
                                    let v = interp_v.clamp(0.0, 1.0);

                                    let tex_x = (u * (img_w as f32 - 1.0)) as u32;
                                    let tex_y = ((1.0 - v) * (img_h as f32 - 1.0)) as u32;

                                    if tex_x < img_w && tex_y < img_h {
                                        let pixel = current_texture.get_pixel(tex_x, tex_y);
                                        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
                                        if a > 0 {
                                            let final_alpha = (a as f32 / 255.0) * opacity;
                                            if final_alpha > 0.1 {
                                                let luminance = 0.299 * (r as f32)
                                                    + 0.587 * (g as f32)
                                                    + 0.114 * (b as f32);

                                                let char_index = (luminance / 255.0
                                                    * (ASCII_CHARS.len() - 1) as f32)
                                                    .round()
                                                    as usize;

                                                let char_index =
                                                    char_index.clamp(0, ASCII_CHARS.len() - 1);
                                                let display_char = ASCII_CHARS[char_index];

                                                context.set_pixel(x, y, display_char, (r, g, b));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            context.flush(true)?;
            stdout().flush()?;
            let elapsed = frame_start.elapsed();
            if elapsed < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed);
            }
        }
        execute!(stdout(), cursor::Show)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }

    fn transform_to_screen(&self, pos: CsmVector2, width: u16, height: u16) -> Vec3 {
        let transformed_x = pos.x * self.scale + self.offset_x;
        let transformed_y = pos.y * self.scale + self.offset_y;

        let mut x = (transformed_x + 1.0) / 2.0;
        let mut y = 1.0 - (transformed_y + 1.0) / 2.0;

        x *= width as f32;
        y *= height as f32;

        Vec3 { x, y, z: 0.0 }
    }

    pub fn find_param_index(&self, target_id: &str) -> Option<usize> {
        unsafe {
            let count = csmGetParameterCount(self.model) as usize;
            let ids_ptr = csmGetParameterIds(self.model);
            if ids_ptr.is_null() {
                return None;
            }

            for i in 0..count {
                let id_ptr = *ids_ptr.add(i);

                if !id_ptr.is_null() {
                    let c_str = CStr::from_ptr(id_ptr);
                    if let Ok(id_str) = c_str.to_str() {
                        if id_str == target_id {
                            return Some(i);
                        }
                    }
                }
            }
        }
        None
    }
}
