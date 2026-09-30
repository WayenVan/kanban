use crate::components::{
    fit_rows, scroll_offset_to_show, slots_height, styled_list_item, ListItemConfig, Popup,
    RowList, Slot,
};
use crate::theme::*;
use ratatui::{layout::Rect, widgets::Paragraph, Frame};

/// Draws a popup holding a list: an optional label, then room for
/// `list_rows` rows. On a short screen the list shrinks to a single row
/// before the label goes. Returns the list's area.
pub fn list_popup_frame(
    frame: &mut Frame,
    title: &str,
    label: Option<&str>,
    list_rows: u16,
    width_percent: u16,
) -> Rect {
    let mut slots = vec![Slot::gap(1)];
    if label.is_some() {
        slots.extend([Slot::line(5), Slot::gap(2)]);
    }
    slots.extend([Slot::flexible(list_rows.max(1), 1, 9), Slot::gap(0)]);

    let area = Popup::new(title)
        .width_percent(width_percent)
        .border_style(focused_border())
        .content_height(slots_height(&slots))
        .render(frame);
    let rows = fit_rows(area, &slots);

    if let (Some(label), Some(row)) = (label, rows[1]) {
        frame.render_widget(Paragraph::new(label).style(highlight_text()), row);
    }
    rows[slots.len() - 2].unwrap_or(Rect { height: 0, ..area })
}

/// A list popup over `rows`, scrolled to keep the selected row in view.
pub fn render_list_popup(
    frame: &mut Frame,
    title: &str,
    label: Option<&str>,
    rows: RowList,
    width_percent: u16,
) {
    let total = rows.len();
    let area = list_popup_frame(frame, title, label, total as u16, width_percent);
    let scroll = scroll_offset_to_show(rows.selected().unwrap_or(0), total, area.height as usize);
    frame.render_widget(rows.focused(true).scroll(scroll as u16), area);
}

/// A list popup built from `items`: `format_fn` gives each item's text and
/// an optional suffix; the `active_idx` item is marked as the current one.
#[allow(clippy::too_many_arguments)]
pub fn render_selection_popup_with_lines<'a, I, F>(
    frame: &mut Frame,
    title: &str,
    label: Option<&str>,
    items: I,
    format_fn: F,
    selected_idx: Option<usize>,
    active_idx: Option<usize>,
    width_percent: u16,
) where
    I: IntoIterator,
    I::Item: 'a,
    F: Fn(usize, &I::Item, bool, bool) -> (String, Option<String>),
{
    let mut rows = RowList::new();
    for (idx, item) in items.into_iter().enumerate() {
        let is_selected = selected_idx == Some(idx);
        let is_active = active_idx == Some(idx);
        let (mut text, suffix) = format_fn(idx, &item, is_selected, is_active);
        if let Some(suffix) = suffix {
            text.push_str(&suffix);
        }
        let config = ListItemConfig::new().active(is_active);
        rows.push_row(styled_list_item(text, &config), is_selected);
    }
    render_list_popup(frame, title, label, rows, width_percent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, text::Line, Terminal};

    fn render(width: u16, height: u16, items: usize, selected: usize) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                let mut rows = RowList::new();
                for i in 0..items {
                    rows.push_row(Line::from(format!("item-{i}")), i == selected);
                }
                render_list_popup(frame, "Pick", Some("Choose one:"), rows, 60);
            })
            .unwrap();
        let buffer = terminal.backend().buffer().clone();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_list_popup_is_as_tall_as_its_rows() {
        let grid = render(80, 40, 4, 0);
        let rows_with_border = grid.lines().filter(|l| l.contains('│')).count();
        // pad + label + gap + 4 items + pad
        assert_eq!(rows_with_border, 8, "{grid}");
    }

    #[test]
    fn test_list_popup_on_a_short_screen_keeps_the_selected_item_visible() {
        let grid = render(80, 5, 20, 15);
        assert!(grid.contains("item-15"), "{grid}");
    }
}
