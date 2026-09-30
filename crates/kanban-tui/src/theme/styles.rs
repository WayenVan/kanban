use super::colors::*;
use kanban_domain::{CardPriority, SprintStatus};
use ratatui::style::{Color, Modifier, Style};

pub fn ended_marker() -> Style {
    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
}

pub fn focused_border() -> Style {
    Style::default().fg(FOCUSED_BORDER)
}

pub fn unfocused_border() -> Style {
    Style::default().fg(UNFOCUSED_BORDER)
}

pub fn deleted_view_focused_border() -> Style {
    Style::default().fg(ratatui::style::Color::Yellow)
}

/// `style` on the selected row: the background changes, and dimmed text is
/// lifted a step so it stays readable on it; other colours are kept.
pub fn on_selection(style: Style, focused: bool) -> Style {
    let style = style.bg(selected_bg(focused));
    if style.fg == Some(Color::DarkGray) {
        style.fg(selected_dim_text())
    } else {
        style
    }
}

pub fn active_item() -> Style {
    Style::default()
        .fg(ACTIVE_ITEM)
        .add_modifier(Modifier::BOLD)
}

pub fn normal_text() -> Style {
    Style::default().fg(NORMAL_TEXT)
}

pub fn error_text() -> Style {
    Style::default().fg(ERROR_COLOR)
}

pub fn label_text() -> Style {
    Style::default().fg(LABEL_TEXT)
}

pub fn highlight_text() -> Style {
    Style::default().fg(HIGHLIGHT_TEXT)
}

pub fn bold_highlight() -> Style {
    Style::default()
        .fg(HIGHLIGHT_TEXT)
        .add_modifier(Modifier::BOLD)
}

pub fn priority_style(priority: CardPriority) -> Style {
    let color = match priority {
        CardPriority::Critical => PRIORITY_CRITICAL,
        CardPriority::High => PRIORITY_HIGH,
        CardPriority::Medium => PRIORITY_MEDIUM,
        CardPriority::Low => PRIORITY_LOW,
    };
    Style::default().fg(color)
}

pub fn points_style(points: u8) -> Style {
    let color = match points {
        1 => POINTS_1,
        2 => POINTS_2,
        3 => POINTS_3,
        4 => POINTS_4,
        5 => POINTS_5,
        _ => NORMAL_TEXT,
    };
    Style::default().fg(color).add_modifier(Modifier::BOLD)
}

pub fn sprint_status_style(status: SprintStatus) -> Style {
    let color = match status {
        SprintStatus::Active => STATUS_ACTIVE,
        SprintStatus::Planning => STATUS_PLANNING,
        SprintStatus::Completed => STATUS_COMPLETED,
        SprintStatus::Cancelled => STATUS_CANCELLED,
    };
    Style::default().fg(color)
}

pub fn popup_bg() -> Style {
    Style::default().bg(POPUP_BG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_selection_lifts_dim_text_and_keeps_other_colours() {
        let dim = on_selection(Style::default().fg(Color::DarkGray), true);
        assert_eq!(dim.fg, Some(selected_dim_text()));
        assert_eq!(dim.bg, Some(selected_bg(true)));

        let yellow = on_selection(Style::default().fg(Color::Yellow), true);
        assert_eq!(yellow.fg, Some(Color::Yellow));
    }
}
