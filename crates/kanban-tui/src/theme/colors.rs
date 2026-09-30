use std::sync::OnceLock;

use ratatui::style::Color;

pub const FOCUSED_BORDER: Color = Color::Cyan;
pub const UNFOCUSED_BORDER: Color = Color::DarkGray;

pub const ACTIVE_ITEM: Color = Color::Green;
pub const DONE_TEXT: Color = Color::DarkGray;
pub const NORMAL_TEXT: Color = Color::White;
pub const LABEL_TEXT: Color = Color::DarkGray;
pub const HIGHLIGHT_TEXT: Color = Color::Yellow;
pub const MULTI_SELECT_MARKER: Color = Color::Magenta;

pub const PRIORITY_CRITICAL: Color = Color::Red;
pub const PRIORITY_HIGH: Color = Color::LightRed;
pub const PRIORITY_MEDIUM: Color = Color::Yellow;
pub const PRIORITY_LOW: Color = Color::White;

pub const POINTS_1: Color = Color::Cyan;
pub const POINTS_2: Color = Color::Green;
pub const POINTS_3: Color = Color::Yellow;
pub const POINTS_4: Color = Color::LightMagenta;
pub const POINTS_5: Color = Color::Red;

pub const STATUS_ACTIVE: Color = Color::Green;
pub const STATUS_PLANNING: Color = Color::Yellow;
pub const STATUS_COMPLETED: Color = Color::Gray;
pub const STATUS_CANCELLED: Color = Color::Red;

pub const POPUP_BG: Color = Color::Black;
pub const ERROR_COLOR: Color = Color::Red;

pub const FLASH_DELETE: Color = Color::Red;
pub const FLASH_RESTORE: Color = Color::Blue;

/// Selection backgrounds: a surface one step off the terminal's own
/// background (Catppuccin surface1 / surface0), so every span on the
/// selected row keeps its colour.
const SELECTED_BG_DARK: Color = Color::Rgb(0x45, 0x47, 0x5a);
const SELECTED_BG_DARK_UNFOCUSED: Color = Color::Rgb(0x31, 0x32, 0x44);
const SELECTED_BG_LIGHT: Color = Color::Rgb(0xbc, 0xc0, 0xcc);
const SELECTED_BG_LIGHT_UNFOCUSED: Color = Color::Rgb(0xcc, 0xd0, 0xda);

/// Dimmed text (DarkGray) on the selected row is lifted to this (Catppuccin
/// subtext0): terminal themes often make DarkGray nearly the selection surface.
const SELECTED_DIM_TEXT_DARK: Color = Color::Rgb(0xa6, 0xad, 0xc8);
const SELECTED_DIM_TEXT_LIGHT: Color = Color::Rgb(0x6c, 0x6f, 0x85);

static LIGHT_BACKGROUND: OnceLock<bool> = OnceLock::new();

/// Asks the terminal for its background colour (OSC 11). Call once, before
/// raw mode and the alternate screen; a terminal that does not answer is
/// treated as dark.
pub fn detect_terminal_background() {
    LIGHT_BACKGROUND.get_or_init(|| {
        use terminal_colorsaurus::{theme_mode, QueryOptions, ThemeMode};
        matches!(theme_mode(QueryOptions::default()), Ok(ThemeMode::Light))
    });
}

fn light_background() -> bool {
    LIGHT_BACKGROUND.get().copied().unwrap_or(false)
}

/// Background of the selected row; fainter while its panel lacks focus.
pub fn selected_bg(focused: bool) -> Color {
    match (light_background(), focused) {
        (false, true) => SELECTED_BG_DARK,
        (false, false) => SELECTED_BG_DARK_UNFOCUSED,
        (true, true) => SELECTED_BG_LIGHT,
        (true, false) => SELECTED_BG_LIGHT_UNFOCUSED,
    }
}

/// Foreground that replaces DarkGray on the selected row.
pub fn selected_dim_text() -> Color {
    if light_background() {
        SELECTED_DIM_TEXT_LIGHT
    } else {
        SELECTED_DIM_TEXT_DARK
    }
}
