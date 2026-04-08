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
use ratatui::{Terminal, backend::CrosstermBackend, widgets::Paragraph};

use crate::context::*;
use crate::effect::eye_blink::*;
use crate::effect::pose::*;
use crate::expression::exp::*;
use crate::expression::manager::*;
use crate::ffi::*;
use crate::geometry::*;
use crate::model::*;
use crate::model_setting::ModelSetting;
use crate::motion::amotion::*;
use crate::motion::json::*;
use crate::motion::manager::*;
use crate::ui::*;

pub struct Renderer {
    pub count: usize,
    pub model: Model,
    constant_flags: *const u8,
    texture_indices: *const i32,
    vertex_counts: *const i32,
    vertex_positions: *const *const CsmVector2,
    vertex_uvs: *const *const CsmVector2,
    index_counts: *const i32,
    indices: *const *const u16,
    multiply_colors: *const CsmVector4,
    screen_colors: *const CsmVector4,
    shader: Box<[char]>,

    mask_counts: *const i32,
    masks: *const *const i32,

    textures: Vec<DynamicImage>,
    offset_x: f32,
    offset_y: f32,
    scale: f32,
    start_time: Instant,
}

impl Renderer {
    pub fn new(model_ptr: *mut CsmModel, textures: Vec<DynamicImage>, shader: Box<[char]>) -> Self {
        let model = Model::new(model_ptr);
        unsafe {
            Self {
                model,
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
                shader,

                mask_counts: csmGetDrawableMaskCounts(model_ptr),
                masks: csmGetDrawableMasks(model_ptr),

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
        mm: &mut MotionManager,
        model_setting: &mut ModelSetting,
        em: &mut ExpressionManager,
        pose: &mut Option<Pose>,
    ) -> Result<(), Box<dyn Error>> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), cursor::Hide)?;

        // terminal
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;

        let fps = 60.0;
        let target_frame_time = Duration::from_secs_f64(1.0 / fps);
        let mut last_frame = Instant::now();

        // get eye_blink
        let mut eye_blink = EyeBlink::new(model_setting);

        //        if let Some(exp) = exp {
        //            em.qm.start_motion(exp, false);
        //        }
        //
        if let Some(pose) = pose {
            pose.reset(&mut self.model);
        }

        let mut mask_buffer = vec![false; (context.width as usize) * (context.height as usize)];

        loop {
            let frame_start = Instant::now();

            if event::poll(Duration::from_millis(1))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                    match context.current_panel {
                        Panel::None => match code {
                            KeyCode::Char('q') => break,
                            KeyCode::Up => self.offset_y -= 0.1,
                            KeyCode::Down => self.offset_y += 0.1,
                            KeyCode::Left => self.offset_x -= 0.1,
                            KeyCode::Right => self.offset_x += 0.1,
                            KeyCode::Char('=') | KeyCode::Char('+') => self.scale *= 1.1,
                            KeyCode::Char('-') => self.scale *= 0.9,
                            KeyCode::Char('m') => {
                                context.current_panel = Panel::Op;
                                context.current_op_panel = OpPanel::Motions;
                            }
                            KeyCode::Char('p') => {
                                context.current_panel = Panel::Debug;
                                if let DebugPanel::None = context.current_debug_panel {
                                    context.current_debug_panel = DebugPanel::Parameters;
                                }
                            }

                            _ => {}
                        },
                        Panel::Op => match context.current_op_panel {
                            OpPanel::Motions => match code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    context.current_panel = Panel::None;
                                    context.current_op_panel = OpPanel::None;
                                }
                                KeyCode::Up => context.motion_list_state.select_previous(),
                                KeyCode::Down => context.motion_list_state.select_next(),
                                KeyCode::Enter => {
                                    if let Some(idx) = context.motion_list_state.selected() {
                                        let file = model_setting.get_all_motion_names()[idx];
                                        let motion_data =
                                            MotionData::from_path(&context.base_dir, file)?;
                                        let motion = CubismMotion::new(motion_data);
                                        mm.start_motion_priority(motion, true, 0);
                                    }
                                    if let Some(p) = pose {
                                        p.reset(&mut self.model);
                                    }
                                }
                                KeyCode::Char('m') => {
                                    context.current_panel = Panel::Op;
                                    context.current_op_panel = OpPanel::Motions;
                                }
                                KeyCode::Char('p') => {
                                    context.current_panel = Panel::Debug;
                                    if let DebugPanel::None = context.current_debug_panel {
                                        context.current_debug_panel = DebugPanel::Parameters;
                                    }
                                }
                                _ => {}
                            },
                            OpPanel::None => {}
                        },
                        Panel::Debug => match context.current_debug_panel {
                            DebugPanel::Parameters | DebugPanel::PartOpacities => match code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    context.current_panel = Panel::None;
                                    context.current_debug_panel = DebugPanel::None;
                                }
                                KeyCode::Char('1') => {
                                    context.current_debug_panel = DebugPanel::Parameters;
                                }
                                KeyCode::Char('2') => {
                                    context.current_debug_panel = DebugPanel::PartOpacities;
                                }
                                KeyCode::Up => context.param_list_state.select_previous(),
                                KeyCode::Down => context.param_list_state.select_next(),
                                KeyCode::Char('m') => {
                                    context.current_panel = Panel::Op;
                                    context.current_op_panel = OpPanel::Motions;
                                }
                                _ => {}
                            },
                            DebugPanel::None => {}
                        },
                    }
                }
            }
            context.update()?;
            context.clear();
            let needed = (context.width as usize) * (context.height as usize);
            if mask_buffer.len() != needed {
                mask_buffer.resize(needed, false);
            }
            mask_buffer.fill(false);
            self.model.load_parameters();

            let delta_time = last_frame.elapsed().as_secs_f32();
            last_frame = Instant::now();

            mm.update_motion(&mut self.model, delta_time);
            //            em.update_motion(&mut self.model, delta_time);
            eye_blink.update_parameters(&mut self.model, delta_time);

            if let Some(pose) = pose {
                pose.update_parameters(&mut self.model, delta_time);
            }

            self.model.save_parameters();

            // applying manioulation to Drawable
            unsafe {
                csmResetDrawableDynamicFlags(self.model.model);
                csmUpdateModel(self.model.model);
            }

            let (dy_flags, opacities, vt_positions, render_orders) = unsafe {
                let dy_flags = csmGetDrawableDynamicFlags(self.model.model);
                let opacities = csmGetDrawableOpacities(self.model.model);
                let vt_positions = csmGetDrawableVertexPositions(self.model.model);
                let render_orders = csmGetDrawableRenderOrders(self.model.model);

                self.multiply_colors = csmGetDrawableMultiplyColors(self.model.model);
                self.screen_colors = csmGetDrawableScreenColors(self.model.model);

                (dy_flags, opacities, vt_positions, render_orders)
            };

            let mut drawables: Vec<usize> = (0..self.count).collect();
            drawables.sort_by_key(|&i| unsafe { *render_orders.add(i) });

            for &drawable_idx in &drawables {
                unsafe {
                    let flags = *dy_flags.add(drawable_idx);
                    let is_visible = (flags & 1) != 0;
                    if !is_visible {
                        continue;
                    }
                    let opacity = *opacities.add(drawable_idx);
                    if opacity <= 0.001 {
                        continue;
                    }

                    let mask_count = *self.mask_counts.add(drawable_idx) as usize;
                    let has_mask = mask_count > 0;

                    // --- Simple MASK operation ---
                    if has_mask {
                        mask_buffer.fill(false);
                        let mask_indices_ptr = *self.masks.add(drawable_idx);

                        for m in 0..mask_count {
                            let mask_idx = *mask_indices_ptr.add(m) as usize;

                            let m_index_count = *self.index_counts.add(mask_idx) as usize;
                            let m_indices_ptr = *self.indices.add(mask_idx);
                            let m_vertices_ptr = *vt_positions.add(mask_idx);

                            for i in (0..m_index_count).step_by(3) {
                                let i0 = *m_indices_ptr.add(i) as usize;
                                let i1 = *m_indices_ptr.add(i + 1) as usize;
                                let i2 = *m_indices_ptr.add(i + 2) as usize;

                                let v0 = self.transform_to_screen(
                                    *m_vertices_ptr.add(i0),
                                    context.width,
                                    context.height,
                                );
                                let v1 = self.transform_to_screen(
                                    *m_vertices_ptr.add(i1),
                                    context.width,
                                    context.height,
                                );
                                let v2 = self.transform_to_screen(
                                    *m_vertices_ptr.add(i2),
                                    context.width,
                                    context.height,
                                );

                                let triangle = Triangle::new(v0, v1, v2);
                                let bbox = triangle.get_box();
                                let min_x = bbox.minx.max(0.0) as u16;
                                let max_x = bbox.maxx.min((context.width - 1) as f32) as u16;
                                let min_y = bbox.miny.max(0.0) as u16;
                                let max_y = bbox.maxy.min((context.height - 1) as f32) as u16;

                                let total_area = triangle.signed_area();
                                if total_area == 0.0 {
                                    continue;
                                }

                                for y in min_y..=max_y {
                                    for x in min_x..=max_x {
                                        let p = Vec3 {
                                            x: x as f32,
                                            y: y as f32,
                                            z: 0.0,
                                        };
                                        let w0 =
                                            Triangle::new(v1, v2, p).signed_area() / total_area;
                                        let w1 =
                                            Triangle::new(v2, v0, p).signed_area() / total_area;
                                        let w2 = 1.0 - w0 - w1;

                                        if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                                            mask_buffer[(y as usize) * (context.width as usize)
                                                + (x as usize)] = true;
                                        }
                                    }
                                }
                            }
                        }
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
                        if total_area == 0.0 {
                            continue;
                        }

                        for y in min_y..=max_y {
                            for x in min_x..=max_x {
                                if has_mask
                                    && !mask_buffer
                                        [(y as usize) * (context.width as usize) + (x as usize)]
                                {
                                    continue;
                                }

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
                                                    * (self.shader.len() - 1) as f32)
                                                    .round()
                                                    as usize;
                                                let char_index =
                                                    char_index.clamp(0, self.shader.len() - 1);
                                                let display_char = self.shader[char_index];

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

            // draw ui
            terminal.draw(|f| match ui(f, context, &self.model) {
                Ok(_) => {}
                Err(e) => {
                    println!("{:?}", e);
                }
            })?;

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
            let count = csmGetParameterCount(self.model.model) as usize;
            let ids_ptr = csmGetParameterIds(self.model.model);
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

    pub fn find_part_index(&self, target_id: &str) -> Option<usize> {
        unsafe {
            let count = csmGetPartCount(self.model.model) as usize;
            let ids_ptr = csmGetPartIds(self.model.model);
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
