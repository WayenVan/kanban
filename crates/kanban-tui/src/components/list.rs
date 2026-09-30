use crate::theme::{active_item, normal_text};
use ratatui::{
    style::Style,
    text::{Line, Span},
};

/// How a list row looks apart from the cursor, which [`RowList`] paints.
///
/// [`RowList`]: crate::components::RowList
pub struct ListItemConfig {
    pub is_active: bool,
    pub is_multi_selected: bool,
}

impl ListItemConfig {
    pub fn new() -> Self {
        Self {
            is_active: false,
            is_multi_selected: false,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn multi_selected(mut self, multi_selected: bool) -> Self {
        self.is_multi_selected = multi_selected;
        self
    }

    pub fn item_style(&self) -> Style {
        if self.is_active {
            active_item()
        } else {
            normal_text()
        }
    }

    pub fn item_prefix(&self) -> &'static str {
        if self.is_active {
            "● "
        } else if self.is_multi_selected {
            "► "
        } else {
            "  "
        }
    }
}

impl Default for ListItemConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn styled_list_item(text: impl Into<String>, config: &ListItemConfig) -> Line<'static> {
    let prefix = config.item_prefix();
    let style = config.item_style();
    Line::from(Span::styled(format!("{}{}", prefix, text.into()), style))
}
