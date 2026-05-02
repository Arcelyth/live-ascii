use std::collections::HashMap;
use std::error::Error;
use std::io::stdout;
use std::sync::{
    Arc,
    mpsc::{Sender, Receiver, channel}
};

use ratatui::widgets::ListState;

use crossterm::{
    QueueableCommand, cursor, queue,
    style::{Color, PrintStyledContent, Stylize},
    terminal,
};
use ratatui::style::{Color as RatatuiColor, Style};
use ratatui::text::{Line, Span, Text};

use crate::live::json::*;
use crate::tracker::*;
use crate::model_setting::ModelSetting;
use crate::ui::popup::*;
use crate::receiver::*;

#[derive(Debug)]
pub enum OpPanel {
    None,
    Motions,
}

#[derive(Debug)]
pub enum DebugPanel {
    None,
    Parameters,
    PartOpacities,
    AppliedExp,
    ActionQueue, 
    Camera,
    Manager,
}

#[derive(Debug)]
pub enum Panel {
    None,
    Op,
    Debug,
}


#[derive(Debug)]
pub struct Context {
    pub width: u16,
    pub height: u16,
    // RGB
    pub frame_buffer: Vec<(char, (u8, u8, u8))>,
    pub image: bool,
    pub base_dir: Arc<str>,
    pub model_setting: ModelSetting,
    // motion panel
    pub motion_list_state: ListState,
    // parameter debug panel
    pub param_list_state: ListState,
    // camera debug offset
    pub camera_offset: u16,
    pub context_offset: u16,
    pub current_op_panel: OpPanel,
    pub current_debug_panel: DebugPanel,
    pub current_panel: Panel,
    pub live_setting: Option<Live>,
    pub action_queue: Vec<Action>,
    pub active_expressions: std::collections::HashMap<String, usize>,

    pub tracker: Tracker,
    pub camera: bool,
    pub receiver: Option<MsgReceiver>,
    pub msg_chan: (Sender<String>, Receiver<String>),
    pub use_physics: bool,
    pub popups: Popups,
}

impl Context {
    pub fn new(image: bool, model_setting: ModelSetting, base_dir: &str, camera: bool, tracker: Tracker) -> Self {
        Self {
            width: 0,
            height: 0,
            frame_buffer: vec![],
            image,
            base_dir: base_dir.into(),
            model_setting,
            motion_list_state: ListState::default().with_selected(Some(0)),
            param_list_state: ListState::default().with_selected(Some(0)),
            camera_offset: 0,
            context_offset: 0,
            current_op_panel: OpPanel::None,
            current_debug_panel: DebugPanel::None,
            current_panel: Panel::None,
            live_setting: None,
            action_queue: vec![],
            active_expressions: HashMap::new(),
            tracker,
            camera,
            receiver: None, 
            msg_chan: channel(),
            use_physics: false,
            popups: Popups::new(),
        }
    }

    pub fn set_live_setting(&mut self, live: Live) {
        self.live_setting = Some(live);
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, ch: char, color: (u8, u8, u8)) {
        if x < self.width && y < self.height {
            let idx = x + y * self.width;
            self.frame_buffer[idx as usize] = (ch, color);
        }
    }

    pub fn flush(&self, color: bool) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout();

        if !self.image {
            stdout.queue(cursor::MoveTo(0, 0))?;
        }

        match color {
            false => {
                let frame: String = self.frame_buffer.iter().map(|pixel| pixel.0).collect();
                println!("{}", frame);
            }
            true => {
                for pixel in &self.frame_buffer {
                    let styled = pixel
                        .0
                        .with(Color::Rgb {
                            r: (pixel.1).0,
                            g: (pixel.1).1,
                            b: (pixel.1).2,
                        })
                        .on(Color::Rgb {
                            r: 10,
                            g: 10,
                            b: 10,
                        });
                    queue!(stdout, PrintStyledContent(styled))?;
                }
            }
        }

        Ok(())
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let (tw, th) = terminal::size()?;
        if self.width != tw || self.height != th {
            self.width = tw;
            self.height = th;
            self.frame_buffer
                .resize((tw * th) as usize, (' ', (0, 0, 0)));
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.frame_buffer.fill((' ', (0, 0, 0)));
    }

    pub fn buffer_to_text(&self) -> Text<'static> {
        let mut lines = Vec::with_capacity(self.height as usize);

        for y in 0..self.height {
            let mut spans = Vec::with_capacity(self.width as usize);
            for x in 0..self.width {
                let idx = (y * self.width + x) as usize;
                if let Some((ch, (r, g, b))) = self.frame_buffer.get(idx) {
                    spans.push(Span::styled(
                        ch.to_string(),
                        Style::default().fg(RatatuiColor::Rgb(*r, *g, *b)),
                    ));
                }
            }
            lines.push(Line::from(spans));
        }
        Text::from(lines)
    }

    pub fn get_active_expressions(&self) -> Vec<&str> {
        self.active_expressions.keys().map(|s| s.as_str()).collect()
    }

}
