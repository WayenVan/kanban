use crate::app::App;
use crate::components::{fit_rows, Popup, RowList, Slot};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_manage_parents_popup(app: &App, frame: &mut Frame) {
    render_relationship_popup(app, frame, "Set Parents");
}

pub fn render_manage_children_popup(app: &App, frame: &mut Frame) {
    render_relationship_popup(app, frame, "Set Children");
}

fn render_relationship_popup(app: &App, frame: &mut Frame, title: &str) {
    let area = Popup::new(title)
        .height_percent(70)
        .border_style(crate::theme::focused_border())
        .render(frame);
    let rows = fit_rows(
        area,
        &[
            Slot::gap(0),
            Slot::fixed(3, 7),
            Slot::flexible(area.height, 1, 9),
            Slot::line(5),
            Slot::gap(1),
        ],
    );

    if let Some(row) = rows[1] {
        render_relationship_search_box(app, frame, row);
    }
    if let Some(row) = rows[2] {
        render_relationship_card_list(app, frame, row);
    }
    if let Some(row) = rows[3] {
        render_relationship_instructions(app, frame, row);
    }
}

fn render_relationship_search_box(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let search_border_style = if app.relationship.search_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    let search_block = Block::default()
        .title("Search")
        .borders(Borders::ALL)
        .border_style(search_border_style);

    let search_text: Line = if app.relationship.search_active {
        Line::from(vec![
            Span::styled(&app.relationship.search, Style::default().fg(Color::White)),
            Span::styled("_", Style::default().fg(Color::Yellow)),
        ])
    } else if app.relationship.search.is_empty() {
        Line::from(Span::styled(
            "/ to search",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::styled(
            &app.relationship.search,
            Style::default().fg(Color::White),
        ))
    };

    let search = Paragraph::new(search_text).block(search_block);
    frame.render_widget(search, area);
}

fn render_relationship_card_list(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let filtered_cards: Vec<_> = if app.relationship.search.is_empty() {
        app.relationship.card_ids.clone()
    } else {
        let search_lower = app.relationship.search.to_lowercase();
        app.relationship
            .card_ids
            .iter()
            .filter(|card_id| {
                app.model
                    .card_by_id_state(**card_id)
                    .loaded()
                    .copied()
                    .map(|c| c.title.to_lowercase().contains(&search_lower))
                    .unwrap_or(false)
            })
            .copied()
            .collect()
    };

    let mut lines = RowList::new();
    for (idx, card_id) in filtered_cards.iter().enumerate() {
        if let Some(card) = app.model.card_by_id_state(*card_id).loaded().copied() {
            let is_selected = app.relationship.selection.get() == Some(idx);
            let is_checked = app.relationship.selected.contains(card_id);

            let checkbox = if is_checked { "[✓]" } else { "[ ]" };

            let style = if is_checked {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push_row(
                Span::styled(format!("{} {}", checkbox, card.title), style),
                is_selected,
            );
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No eligible cards found",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let scroll = crate::components::scroll_offset_to_show(
        lines.selected().unwrap_or(0),
        lines.len(),
        area.height as usize,
    );
    frame.render_widget(lines.focused(true).scroll(scroll as u16), area);
}

fn render_relationship_instructions(app: &App, frame: &mut Frame, area: ratatui::layout::Rect) {
    let instructions_text = if app.relationship.search_active {
        "Type to search | Enter/Esc: exit search"
    } else {
        "j/k: navigate | Space: toggle | /: search | Esc: close"
    };
    let instructions =
        Paragraph::new(instructions_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(instructions, area);
}
