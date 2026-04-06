use std::error::Error;

use ratatui::Frame;
use ratatui::layout::{Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::context::*;

pub fn ui(frame: &mut Frame, context: &mut Context) -> Result<(), Box<dyn Error>> {
    let model_text = context.buffer_to_text();
    let model_widget = Paragraph::new(model_text);

    let area = frame.area();
    frame.render_widget(model_widget, area);

    if context.show_motions{
        let list_area = Rect::new(2, 2, 30, 10);

        let items: Vec<ListItem> = context.model_setting.get_all_motion_names()
            .iter()
            .map(|m| ListItem::new(*m))
            .collect();

        let list_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Motion List "),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black),
            )
            .highlight_symbol(">> ");

        frame.render_widget(Clear, list_area);
        frame.render_stateful_widget(list_widget, list_area, &mut context.model_list_state);
    }

    Ok(())
}
