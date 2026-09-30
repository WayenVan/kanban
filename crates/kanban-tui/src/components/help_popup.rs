use crate::app::App;
use crate::components::{fit_rows, ListItemConfig, Popup, RowList, Slot};
use crate::theme::*;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const HELP_TITLE: &str = "Help - Keybindings for Current Context";

fn help_popup() -> Popup<'static> {
    Popup::new(HELP_TITLE)
        .width_percent(80)
        .height_percent(80)
        .border_style(focused_border())
}

/// The help popup's header, list and footer areas within `content`; the
/// list is kept longest on a short screen.
fn help_rows(content: Rect) -> [Option<Rect>; 3] {
    let rows = fit_rows(
        content,
        &[
            Slot::fixed(2, 4),
            Slot::flexible(content.height, 1, 9),
            Slot::fixed(2, 3),
        ],
    );
    [rows[0], rows[1], rows[2]]
}

pub fn help_popup_viewport_height(frame_area: Rect) -> usize {
    let [_, list, _] = help_rows(help_popup().content_area(frame_area));
    list.map_or(0, |list| list.height as usize)
}

pub fn render_help_popup(app: &App, frame: &mut Frame) {
    use crate::keybindings::KeybindingRegistry;

    let content = help_popup().render(frame);
    let [header, list, footer_row] = help_rows(content);

    let provider = KeybindingRegistry::get_provider(app);
    let context = provider.get_context();

    if let Some(header) = header {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    context.name.clone(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ]),
            header,
        );
    }
    let Some(list) = list else {
        return;
    };

    let raw_height = list.height as usize;

    let selected_idx = app.ui_state.help_list.get_selected_index();

    let adjusted_height = app
        .ui_state
        .help_list
        .get_adjusted_viewport_height(raw_height);
    let page_info = app.ui_state.help_list.get_render_info(adjusted_height);

    let mut rows = RowList::new();
    rows.extend(crate::scroll_indicators::render_above_indicator(
        page_info.show_above_indicator,
        page_info.items_above,
        "item",
    ));

    let config = ListItemConfig::new();
    for &i in &page_info.visible_indices {
        let Some(binding) = context.bindings.get(i) else {
            continue;
        };
        let style = config.item_style();
        let line = Line::from(vec![
            Span::styled(config.item_prefix().to_string(), style),
            Span::styled(binding.key.to_string(), Style::default().fg(Color::Yellow)),
            Span::raw(" "),
            Span::styled(binding.description.clone(), style),
        ]);
        rows.push_row(line, selected_idx == Some(i));
    }
    rows.extend(crate::scroll_indicators::render_below_indicator(
        page_info.show_below_indicator,
        page_info.items_below,
        "item",
    ));

    frame.render_widget(rows.focused(true), list);

    let footer = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "j/k or ↑↓: navigate | Enter: activate | ESC or ?: close",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
    ]);
    if let Some(footer_area) = footer_row {
        frame.render_widget(footer, footer_area);
    }
}
