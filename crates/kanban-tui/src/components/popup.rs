use crate::theme::{focused_border, highlight_text, input_field, label_text, popup_bg};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// A bordered popup centred on the screen and sized to its content: as tall
/// as the content asks for, as wide as a share of the screen (never
/// narrower than `min_width`), both clamped to the screen. Popups whose
/// content scrolls take a share of the screen's height instead.
pub struct Popup<'a> {
    title: Line<'a>,
    content_height: u16,
    height_percent: Option<u16>,
    width_percent: u16,
    min_width: u16,
    border_style: Style,
}

impl<'a> Popup<'a> {
    pub fn new(title: impl Into<Line<'a>>) -> Self {
        Self {
            title: title.into(),
            content_height: 1,
            height_percent: None,
            width_percent: 60,
            min_width: 40,
            border_style: Style::default(),
        }
    }

    pub fn content_height(mut self, rows: u16) -> Self {
        self.content_height = rows;
        self
    }

    /// Sizes the popup to a share of the screen's height rather than to
    /// its content, for content that scrolls.
    pub fn height_percent(mut self, percent: u16) -> Self {
        self.height_percent = Some(percent);
        self
    }

    pub fn width_percent(mut self, percent: u16) -> Self {
        self.width_percent = percent;
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// The popup's outer rectangle within `screen`.
    pub fn area(&self, screen: Rect) -> Rect {
        let width = (screen.width * self.width_percent / 100)
            .max(self.min_width)
            .min(screen.width);
        let height = match self.height_percent {
            Some(percent) => screen.height * percent / 100,
            None => self.content_height.saturating_add(2),
        }
        .min(screen.height);
        Rect {
            x: screen.x + (screen.width - width) / 2,
            y: screen.y + (screen.height - height) / 2,
            width,
            height,
        }
    }

    /// The content area [`Popup::render`] returns on `screen`: inside the
    /// border, one column in from each side.
    pub fn content_area(&self, screen: Rect) -> Rect {
        Block::default()
            .borders(Borders::ALL)
            .inner(self.area(screen))
            .inner(Margin::new(1, 0))
    }

    /// Width of the content area [`Popup::render`] will return on `screen`.
    pub fn content_width(&self, screen: Rect) -> u16 {
        self.content_area(screen).width
    }

    /// Draws the popup's frame and returns its [content area](Popup::content_area).
    pub fn render(self, frame: &mut Frame) -> Rect {
        let area = self.area(frame.area());
        let content = self.content_area(frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .style(popup_bg());
        frame.render_widget(block, area);
        content
    }
}

/// Rows `text` takes when wrapped to `width` columns.
pub fn wrapped_height(text: &str, width: u16) -> u16 {
    let width = usize::from(width.max(1));
    text.lines()
        .map(|line| Line::from(line).width().div_ceil(width).max(1))
        .sum::<usize>()
        .try_into()
        .unwrap_or(u16::MAX)
}

/// One entry in a popup's vertical stack of rows, for [`fit_rows`].
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    /// Rows wanted.
    pub height: u16,
    /// Rows it can shrink to before it is dropped.
    pub min: u16,
    /// Slots of lower priority are dropped first when space runs out.
    pub priority: u8,
}

impl Slot {
    /// A one-row slot.
    pub fn line(priority: u8) -> Self {
        Self::fixed(1, priority)
    }

    /// A blank row: it costs nothing to keep and is the first to give way.
    pub fn gap(priority: u8) -> Self {
        Self::flexible(1, 0, priority)
    }

    pub fn fixed(height: u16, priority: u8) -> Self {
        Self {
            height,
            min: height,
            priority,
        }
    }

    pub fn flexible(height: u16, min: u16, priority: u8) -> Self {
        Self {
            height,
            min: min.min(height),
            priority,
        }
    }
}

/// Rows wanted by `slots` when every one gets its full height.
pub fn slots_height(slots: &[Slot]) -> u16 {
    slots.iter().map(|s| s.height).sum()
}

/// Stacks `slots` top to bottom in `area`. Every slot first gets its
/// minimum, dropping (`None`) the lowest-priority ones until those fit; the
/// rows left over then go to the slots that can grow, most important first.
/// So on a short screen blank rows vanish, then flexible slots shrink, then
/// the least important rows are dropped.
pub fn fit_rows(area: Rect, slots: &[Slot]) -> Vec<Option<Rect>> {
    let mut kept: Vec<bool> = vec![true; slots.len()];
    let min_total = |kept: &[bool]| -> u16 {
        slots
            .iter()
            .zip(kept)
            .filter(|(_, k)| **k)
            .map(|(s, _)| s.min)
            .sum()
    };

    let mut by_priority: Vec<usize> = (0..slots.len()).collect();
    by_priority.sort_by_key(|&i| (slots[i].priority, std::cmp::Reverse(i)));
    for i in by_priority {
        if min_total(&kept) <= area.height {
            break;
        }
        kept[i] = false;
    }

    let mut spare = area.height.saturating_sub(min_total(&kept));
    let mut heights: Vec<u16> = slots
        .iter()
        .zip(&kept)
        .map(|(slot, &k)| if k { slot.min } else { 0 })
        .collect();
    let mut by_importance: Vec<usize> = (0..slots.len()).filter(|&i| kept[i]).collect();
    by_importance.sort_by_key(|&i| (std::cmp::Reverse(slots[i].priority), i));
    for i in by_importance {
        let grow = (slots[i].height - slots[i].min).min(spare);
        spare -= grow;
        heights[i] += grow;
    }

    let mut y = area.y;
    heights
        .iter()
        .zip(&kept)
        .map(|(&height, &k)| {
            (k && height > 0).then(|| {
                let rect = Rect { y, height, ..area };
                y += height;
                rect
            })
        })
        .collect()
}

/// A one-row text input. The text scrolls sideways to keep the cursor
/// (`cursor_col`, in display columns) inside the field; when `focused`, the
/// terminal cursor is placed on it.
pub fn render_input_field(
    frame: &mut Frame,
    area: Rect,
    text: &str,
    cursor_col: usize,
    focused: bool,
    text_style: Style,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let row = Rect { height: 1, ..area };
    let cursor_col = u16::try_from(cursor_col).unwrap_or(u16::MAX);
    let offset = cursor_col.saturating_sub(row.width - 1);
    let field = Paragraph::new(text)
        .style(input_field(focused).patch(text_style))
        .scroll((0, offset));
    frame.render_widget(field, row);
    if focused {
        frame.set_cursor_position((row.x + cursor_col - offset, row.y));
    }
}

/// A single-line text prompt: a label above one input row.
pub fn render_input_popup(
    frame: &mut Frame,
    title: &str,
    label: &str,
    input_text: &str,
    cursor_pos: usize,
) {
    let slots = [
        Slot::gap(1),
        Slot::line(3),
        Slot::gap(2),
        Slot::line(4),
        Slot::gap(0),
    ];
    let area = Popup::new(title)
        .border_style(focused_border())
        .content_height(slots_height(&slots))
        .render(frame);
    let rows = fit_rows(area, &slots);

    if let Some(row) = rows[1] {
        frame.render_widget(Paragraph::new(label).style(highlight_text()), row);
    }
    if let Some(row) = rows[3] {
        render_input_field(frame, row, input_text, cursor_pos, true, Style::default());
    }
}

/// A form field's label: highlighted while the field has focus.
pub fn field_label(text: &str, focused: bool) -> Paragraph<'_> {
    let style = if focused {
        highlight_text()
    } else {
        label_text()
    };
    Paragraph::new(text).style(style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn heights(rows: &[Option<Rect>]) -> Vec<Option<u16>> {
        rows.iter().map(|r| r.map(|r| r.height)).collect()
    }

    #[test]
    fn test_fit_rows_with_room_gives_every_slot_its_height() {
        let slots = [Slot::line(0), Slot::fixed(3, 1), Slot::line(2)];
        let rows = fit_rows(Rect::new(0, 5, 10, 20), &slots);
        assert_eq!(heights(&rows), vec![Some(1), Some(3), Some(1)]);
        assert_eq!(rows[2].unwrap().y, 9);
    }

    #[test]
    fn test_fit_rows_without_room_drops_the_lowest_priority_first() {
        let slots = [Slot::line(1), Slot::line(3), Slot::line(0), Slot::line(4)];
        let rows = fit_rows(Rect::new(0, 0, 10, 2), &slots);
        assert_eq!(heights(&rows), vec![None, Some(1), None, Some(1)]);
        assert_eq!(rows[3].unwrap().y, 1);
    }

    #[test]
    fn test_fit_rows_shrinks_flexible_slots_before_dropping_any() {
        let slots = [Slot::line(0), Slot::flexible(10, 3, 5), Slot::line(1)];
        let rows = fit_rows(Rect::new(0, 0, 10, 6), &slots);
        assert_eq!(heights(&rows), vec![Some(1), Some(4), Some(1)]);
    }

    #[test]
    fn test_fit_rows_on_a_short_screen_gives_up_gaps_before_shrinking_a_list() {
        let slots = [
            Slot::gap(1),
            Slot::line(5),
            Slot::gap(2),
            Slot::flexible(4, 1, 9),
            Slot::gap(0),
        ];
        let rows = fit_rows(Rect::new(0, 0, 10, 6), &slots);
        assert_eq!(heights(&rows), vec![None, Some(1), Some(1), Some(4), None]);
    }

    #[test]
    fn test_wrapped_height_counts_each_line_and_its_wraps() {
        assert_eq!(wrapped_height("abcdef\n\nxy", 4), 4);
    }

    #[test]
    fn test_popup_area_is_clamped_to_a_small_screen() {
        let popup = Popup::new("t").content_height(30);
        let area = popup.area(Rect::new(0, 0, 30, 10));
        assert_eq!(area, Rect::new(0, 0, 30, 10));
    }

    fn render_prompt(width: u16, height: u16, text: &str, cursor: usize) -> (String, (u16, u16)) {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| render_input_popup(frame, "Rename", "Name:", text, cursor))
            .unwrap();
        let cursor = terminal.get_cursor_position().unwrap();
        let buffer = terminal.backend().buffer().clone();
        let grid = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        (grid, (cursor.x, cursor.y))
    }

    #[test]
    fn test_input_popup_on_a_tiny_screen_keeps_the_input_row() {
        let (grid, cursor) = render_prompt(40, 3, "hello", 5);
        assert!(grid.contains("hello"), "{grid}");
        assert_eq!(cursor.1, 1, "{grid}");
    }

    #[test]
    fn test_input_popup_on_a_normal_screen_shows_label_and_input() {
        let (grid, _) = render_prompt(80, 24, "hello", 5);
        let label_row = grid.lines().position(|l| l.contains("Name:")).unwrap();
        let input_row = grid.lines().position(|l| l.contains("hello")).unwrap();
        assert!(input_row > label_row, "{grid}");
    }

    #[test]
    fn test_long_input_scrolls_to_keep_the_cursor_inside_the_field() {
        let text = "x".repeat(100) + "END";
        let (grid, cursor) = render_prompt(40, 10, &text, 103);
        assert!(grid.contains("END"), "{grid}");
        assert!(
            cursor.0 < 39,
            "cursor {cursor:?} outside the popup:\n{grid}"
        );
    }
}
