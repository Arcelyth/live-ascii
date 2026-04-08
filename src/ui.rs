use std::error::Error;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::context::*;
use crate::model::Model;

pub fn ui(frame: &mut Frame, context: &mut Context, model: &Model) -> Result<(), Box<dyn Error>> {
    let model_text = context.buffer_to_text();
    let model_widget = Paragraph::new(model_text);

    let area = frame.area();
    frame.render_widget(model_widget, area);

    let motion_list_border_fg = Color::Magenta;
    let motion_list_border_hl_bg = Color::LightMagenta;
    let motion_list_border_hl_fg = Color::White;

    let param_list_border_fg = Color::Rgb(217, 147, 61);
    let param_list_border_hl_bg = Color::Rgb(199, 188, 137);
    let param_list_border_hl_fg = Color::White;

    let selected_border = Color::Rgb(241, 243, 195);
    match context.current_op_panel {
        OpPanel::Motions => {
            let border_fg = if let Panel::Op = context.current_panel {
                selected_border
            } else {
                motion_list_border_fg
            };
            let list_area = Rect::new(2, 2, 36, 15);

            let items: Vec<ListItem> = context
                .model_setting
                .get_all_motion_names()
                .iter()
                .map(|m| ListItem::new(*m))
                .collect();

            let list_widget = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_fg))
                        .style(
                            Style::default()
                                .fg(motion_list_border_fg)
                                .add_modifier(Modifier::BOLD),
                        )
                        .title(" Motion List "),
                )
                .highlight_style(
                    Style::default()
                        .bg(motion_list_border_hl_bg)
                        .fg(motion_list_border_hl_fg),
                )
                .highlight_symbol("> ");

            frame.render_widget(Clear, list_area);
            frame.render_stateful_widget(list_widget, list_area, &mut context.motion_list_state);
        }
        _ => {}
    }

    match context.current_debug_panel {
        DebugPanel::Parameters => {
            let list_area = Rect::new(2, 20, 45, 20);
            let border_fg = if let Panel::Debug = context.current_panel {
                selected_border
            } else {
                param_list_border_fg
            };

            let items: Vec<ListItem> = model
                .get_all_parameters()
                .iter()
                .map(|m| ListItem::new(format!("{:35}{:.4}", m.0, m.1)))
                .collect();

            let list_widget = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_fg))
                        .style(
                            Style::default()
                                .fg(param_list_border_fg)
                                .add_modifier(Modifier::BOLD),
                        )
                        .title(" Parameters "),
                )
                .highlight_style(
                    Style::default()
                        .bg(param_list_border_hl_bg)
                        .fg(param_list_border_hl_fg),
                )
                .highlight_symbol("> ");

            frame.render_widget(Clear, list_area);
            frame.render_stateful_widget(list_widget, list_area, &mut context.param_list_state);
        }
        DebugPanel::PartOpacities => {
            let list_area = Rect::new(2, 20, 45, 20);
            let border_fg = if let Panel::Debug = context.current_panel {
                selected_border
            } else {
                param_list_border_fg
            };

            let items: Vec<ListItem> = model
                .get_all_part_opacities()
                .iter()
                .map(|m| ListItem::new(format!("{:35}{:.4}", m.0, m.1)))
                .collect();

            let list_widget = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_fg))
                        .style(
                            Style::default()
                                .fg(param_list_border_fg)
                                .add_modifier(Modifier::BOLD),
                        )
                        .title(" Part Opacities "),
                )
                .highlight_style(
                    Style::default()
                        .bg(param_list_border_hl_bg)
                        .fg(param_list_border_hl_fg),
                )
                .highlight_symbol("> ");

            frame.render_widget(Clear, list_area);
            frame.render_stateful_widget(list_widget, list_area, &mut context.param_list_state);
        }
        DebugPanel::AppliedExp => {
            let list_area = Rect::new(2, 20, 45, 20);
            let border_fg = if let Panel::Debug = context.current_panel {
                selected_border
            } else {
                param_list_border_fg
            };

            let items: Vec<ListItem> = context
                .get_active_expressions()
                .iter()
                .map(|m| ListItem::new(format!("{}", m)))
                .collect();

            let list_widget = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_fg))
                        .style(
                            Style::default()
                                .fg(param_list_border_fg)
                                .add_modifier(Modifier::BOLD),
                        )
                        .title(" Applied Expressions "),
                )
                .highlight_style(
                    Style::default()
                        .bg(param_list_border_hl_bg)
                        .fg(param_list_border_hl_fg),
                )
                .highlight_symbol("> ");

            frame.render_widget(Clear, list_area);
            frame.render_stateful_widget(list_widget, list_area, &mut context.param_list_state);
        }

        DebugPanel::PressedKeys => {
            let list_area = Rect::new(2, 20, 45, 20);
            let border_fg = if let Panel::Debug = context.current_panel {
                selected_border
            } else {
                param_list_border_fg
            };

            let items: Vec<ListItem> = context
                .get_pressed_keys()
                .iter()
                .map(|m| ListItem::new(format!("{}", m)))
                .collect();

            let list_widget = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_fg))
                        .style(
                            Style::default()
                                .fg(param_list_border_fg)
                                .add_modifier(Modifier::BOLD),
                        )
                        .title(" Applied Expressions "),
                )
                .highlight_style(
                    Style::default()
                        .bg(param_list_border_hl_bg)
                        .fg(param_list_border_hl_fg),
                )
                .highlight_symbol("> ");

            frame.render_widget(Clear, list_area);
            frame.render_stateful_widget(list_widget, list_area, &mut context.param_list_state);
        }

        _ => {}
    }

    Ok(())
}
