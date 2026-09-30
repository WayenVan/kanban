use crate::app::App;
use crate::components::{fit_rows, slots_height, wrapped_height, Popup, Slot};
use crate::theme::*;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

/// A choice popup: a message, the options (a key and what it does), and a
/// closing instruction.
fn render_choice_popup(
    frame: &mut Frame,
    title: &str,
    message: &str,
    options: [(&str, &str); 2],
    instructions: &str,
) {
    let mut lines = Vec::new();
    for (i, (key, effect)) in options.into_iter().enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            key,
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {effect}"),
            label_text(),
        )));
    }

    let popup = Popup::new(title).width_percent(70);
    let message_rows = wrapped_height(message, popup.content_width(frame.area()));
    let slots = [
        Slot::gap(1),
        Slot::flexible(message_rows, 1, 7),
        Slot::gap(2),
        Slot::flexible(lines.len() as u16, 2, 9),
        Slot::gap(3),
        Slot::line(8),
        Slot::gap(0),
    ];
    let area = popup.content_height(slots_height(&slots)).render(frame);
    let rows = fit_rows(area, &slots);

    if let Some(row) = rows[1] {
        frame.render_widget(
            Paragraph::new(message)
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: false }),
            row,
        );
    }
    if let Some(row) = rows[3] {
        frame.render_widget(Paragraph::new(lines), row);
    }
    if let Some(row) = rows[5] {
        frame.render_widget(Paragraph::new(instructions).style(label_text()), row);
    }
}

pub fn render_conflict_resolution_popup(_app: &App, frame: &mut Frame) {
    render_choice_popup(
        frame,
        "File Conflict Detected",
        "The file was modified by another instance.\nChoose how to resolve this conflict:",
        [
            ("(O)verwrite", "Keep your changes and overwrite the file"),
            ("(T)ake theirs", "Discard your changes and reload the file"),
        ],
        "Press O or T to choose, ESC to retry later",
    );
}

pub fn render_external_change_detected_popup(_app: &App, frame: &mut Frame) {
    render_choice_popup(
        frame,
        "External File Change Detected",
        "The file was modified by another instance.\nYou have unsaved changes. Choose an action:",
        [
            ("(R)eload", "Discard your changes and reload the file"),
            ("(K)eep", "Continue with your changes (save will overwrite)"),
        ],
        "Press R or K to choose, ESC to continue",
    );
}
