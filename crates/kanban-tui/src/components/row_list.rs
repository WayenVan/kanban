use crate::theme::select_cell;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

/// One-line rows drawn as a [`Paragraph`], with the selected row highlighted
/// across the full width.
///
/// Selection is a property of the row, not of its spans: rows are built
/// without knowing whether they are selected, and the highlight is painted
/// onto the buffer after the text, so it covers the gaps and the trailing
/// space as well.
#[derive(Default)]
pub struct RowList<'a> {
    lines: Vec<Line<'a>>,
    selected: Option<usize>,
    focused: bool,
    scroll: u16,
    block: Option<Block<'a>>,
}

impl<'a> RowList<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, line: impl Into<Line<'a>>) {
        self.lines.push(line.into());
    }

    /// Pushes a row, marking it as the selected one when `selected` holds.
    pub fn push_row(&mut self, line: impl Into<Line<'a>>, selected: bool) {
        if selected {
            self.selected = Some(self.lines.len());
        }
        self.push(line);
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Selects the row at `index` among the rows pushed so far.
    pub fn select(mut self, index: Option<usize>) -> Self {
        self.selected = index;
        self
    }

    /// Whether the list's panel has focus; the highlight is fainter without.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn scroll(mut self, offset: u16) -> Self {
        self.scroll = offset;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn lines(&self) -> &[Line<'a>] {
        &self.lines
    }
}

impl<'a> Extend<Line<'a>> for RowList<'a> {
    fn extend<I: IntoIterator<Item = Line<'a>>>(&mut self, iter: I) {
        self.lines.extend(iter);
    }
}

impl<'a> From<Vec<Line<'a>>> for RowList<'a> {
    fn from(lines: Vec<Line<'a>>) -> Self {
        Self {
            lines,
            ..Self::default()
        }
    }
}

impl Widget for RowList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match self.block {
            Some(block) => {
                let inner = block.inner(area);
                block.render(area, buf);
                inner
            }
            None => area,
        };

        let row = self
            .selected
            .and_then(|index| u16::try_from(index).ok()?.checked_sub(self.scroll))
            .filter(|&offset| offset < inner.height)
            .map(|offset| Rect {
                y: inner.y + offset,
                height: 1,
                ..inner
            });

        // The row's background before its text is drawn: cells the text
        // leaves at it get the selection surface, cells a span gave a
        // background of its own (an animation flash) keep theirs.
        let underlay: Vec<Color> = row
            .map(|row| {
                (row.left()..row.right())
                    .map(|x| buf[(x, row.y)].bg)
                    .collect()
            })
            .unwrap_or_default();

        Paragraph::new(self.lines)
            .scroll((self.scroll, 0))
            .render(inner, buf);

        if let Some(row) = row {
            for (x, bg) in (row.left()..row.right()).zip(underlay) {
                let cell = &mut buf[(x, row.y)];
                if cell.bg == bg {
                    select_cell(cell, self.focused);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{selected_bg, selected_dim_text};
    use ratatui::style::{Style, Stylize};
    use ratatui::text::Span;

    fn render(list: RowList, width: u16, height: u16) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);
        list.render(area, &mut buf);
        buf
    }

    #[test]
    fn test_selected_row_is_highlighted_across_the_full_width() {
        let mut list = RowList::new();
        list.push("a");
        list.push_row(Line::from(vec![Span::raw("b"), Span::raw("  c")]), true);
        let buf = render(list.focused(true), 8, 2);

        for x in 0..8 {
            assert_eq!(buf[(x, 1)].bg, selected_bg(true), "column {x}");
            assert_eq!(buf[(x, 0)].bg, Color::Reset, "column {x}");
        }
    }

    #[test]
    fn test_selected_row_lifts_dim_text_and_keeps_other_colours() {
        let mut list = RowList::new();
        list.push_row(
            Line::from(vec![Span::raw("a").dark_gray(), Span::raw("b").yellow()]),
            true,
        );
        let buf = render(list, 4, 1);

        assert_eq!(buf[(0, 0)].fg, selected_dim_text());
        assert_eq!(buf[(1, 0)].fg, Color::Yellow);
        assert_eq!(buf[(1, 0)].bg, selected_bg(false));
    }

    #[test]
    fn test_selected_row_keeps_a_span_background() {
        let mut list = RowList::new();
        list.push_row(Span::styled("x", Style::default().bg(Color::Red)), true);
        let buf = render(list, 3, 1);

        assert_eq!(buf[(0, 0)].bg, Color::Red);
        assert_eq!(buf[(1, 0)].bg, selected_bg(false));
    }

    #[test]
    fn test_scrolled_selection_is_highlighted_inside_the_block() {
        let lines: Vec<Line> = (0..5).map(|i| Line::from(i.to_string())).collect();
        let list = RowList::from(lines)
            .select(Some(3))
            .scroll(2)
            .block(Block::bordered());
        let buf = render(list, 5, 4);

        // Row 3 scrolled by 2 sits on the second inner row (y = 2).
        assert_eq!(buf[(0, 2)].bg, Color::Reset, "border cell");
        assert_eq!(buf[(1, 2)].bg, selected_bg(false));
        assert_eq!(buf[(3, 2)].bg, selected_bg(false));
        assert_eq!(buf[(4, 2)].bg, Color::Reset, "border cell");
        assert_eq!(buf[(1, 1)].bg, Color::Reset);
    }

    #[test]
    fn test_selection_scrolled_out_of_view_paints_nothing() {
        let lines: Vec<Line> = (0..5).map(|i| Line::from(i.to_string())).collect();
        let buf = render(RowList::from(lines).select(Some(0)).scroll(1), 3, 2);
        assert!(buf.content.iter().all(|c| c.bg == Color::Reset));
    }
}
