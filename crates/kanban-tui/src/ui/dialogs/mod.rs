mod boards;
mod cards;
mod columns;
mod sprints;

pub(super) use boards::*;
pub(super) use cards::*;
pub(super) use columns::*;
pub(super) use sprints::*;

use crate::components::*;
use crate::theme::*;
use ratatui::{
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
    Frame,
};

/// Shared modal confirmation popup used by the delete dialogs: a centered
/// bordered box with a yellow body message and the fixed key hint. Kept in one
/// place so the two (board/column) delete confirmations cannot drift.
pub(crate) fn render_confirm_popup(frame: &mut Frame, title: &str, body: String) {
    let popup = Popup::new(title.to_string());
    let body_rows = wrapped_height(&body, popup.content_width(frame.area()));
    let slots = [
        Slot::gap(1),
        Slot::flexible(body_rows, 1, 9),
        Slot::gap(2),
        Slot::line(5),
        Slot::gap(0),
    ];
    let area = popup.content_height(slots_height(&slots)).render(frame);
    let rows = fit_rows(area, &slots);
    if let Some(row) = rows[1] {
        frame.render_widget(
            Paragraph::new(body)
                .style(Style::default().fg(Color::Yellow))
                .wrap(Wrap { trim: false }),
            row,
        );
    }
    if let Some(row) = rows[3] {
        frame.render_widget(
            Paragraph::new("Press ENTER/y to delete, n/ESC to cancel").style(label_text()),
            row,
        );
    }
}
