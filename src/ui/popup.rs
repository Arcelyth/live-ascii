use ratatui::style::Color;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Popup {
    pub content: Arc<str>,
    pub duration: Duration,
    pub created_at: Instant,
    pub position: Option<(usize, usize)>,
    pub size: (usize, usize),
    pub color: Color,
}

impl Popup {
    pub fn new(
        content: impl Into<Arc<str>>,
        duration: Duration,
        size: (usize, usize),
        color: Color,
    ) -> Self {
        Self {
            content: content.into(),
            duration,
            created_at: Instant::now(),
            position: None,
            size,
            color,
        }
    }

    pub fn with_position(mut self, pos: (usize, usize)) -> Self {
        self.position = Some(pos);
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

#[derive(Debug)]
pub struct Popups {
    pub inner: Vec<Popup>,
}

impl Popups {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, popup: Popup) {
        self.inner.push(popup);
    }

    pub fn update(&mut self) {
        self.inner.retain(|p| !p.is_expired());
    }

    pub fn push_err(&mut self, text: &str) {
        self.inner.push(Popup::new(
            text,
            Duration::from_secs(3),
            (text.len() + 3, 3),
            Color::Rgb(230, 119, 119),
        ));
    }

    pub fn push_msg(&mut self, text: &str) {
        self.inner.push(Popup::new(
            text,
            Duration::from_secs(4),
            (text.len() + 3, 3),
            Color::Rgb(128, 242, 176),
        ));
    }

}
