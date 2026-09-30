use crate::app::App;
use crate::components::RowList;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    Frame,
};

pub fn render_settings_view(app: &App, frame: &mut Frame, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(columns[0]);

    let config_location = kanban_service::config::effective_configuration_location(&app.app_config);

    render_settings_configuration(app, frame, left_sections[0], &config_location);
    render_settings_config_file(app, frame, left_sections[1], &config_location);
    render_settings_storage(app, frame, columns[1]);
}

fn render_settings_configuration(app: &App, frame: &mut Frame, area: Rect, config_location: &str) {
    use crate::app::SettingsFocus;
    use crate::components::detail_view::{metadata_line, metadata_line_styled, FieldSectionConfig};

    let config_focused = app.focus.settings_focus == SettingsFocus::Configuration;
    let config_section = FieldSectionConfig::new(" Configuration ")
        .with_focus_indicator(" Configuration [1] ")
        .focused(config_focused);
    let config_block = config_section.block();
    let is_config_selected =
        |i: usize| config_focused && app.selection.settings_config.is_selected(i);
    let mut config_lines = RowList::new();
    for (i, (label, value)) in [
        (
            "Configuration Format",
            app.app_config.effective_configuration_format(),
        ),
        ("Configuration Location", config_location),
        (
            "Default Card Prefix",
            app.app_config.effective_default_card_prefix(),
        ),
        (
            "Default Sprint Prefix",
            app.app_config.effective_default_sprint_prefix(),
        ),
        ("Editing Format", app.app_config.effective_editing_format()),
    ]
    .into_iter()
    .enumerate()
    {
        config_lines.push_row(metadata_line(label, value), is_config_selected(i));
    }
    if app.has_data_file {
        let active_storage_location =
            kanban_service::config::resolve_storage_location(&app.app_config);
        if app.cli_file_override {
            let storage_configured =
                app.original_storage_backend.is_some() || app.original_storage_location.is_some();
            if storage_configured {
                config_lines.push(metadata_line_styled(
                    "Storage Backend",
                    &app.config_storage_backend,
                    Style::default().fg(Color::DarkGray),
                ));
                config_lines.push(metadata_line_styled(
                    "Storage Location",
                    &app.config_storage_location,
                    Style::default().fg(Color::DarkGray),
                ));
            }
            config_lines.push_row(
                metadata_line(
                    "Active Storage Backend",
                    app.app_config.effective_storage_backend(),
                ),
                is_config_selected(7),
            );
            config_lines.push_row(
                metadata_line("Active Storage Location", &active_storage_location),
                is_config_selected(8),
            );
        } else if app.cli_file_provided {
            config_lines.push_row(
                metadata_line(
                    "Active Storage Backend",
                    app.app_config.effective_storage_backend(),
                ),
                is_config_selected(5),
            );
            config_lines.push_row(
                metadata_line("Active Storage Location", &active_storage_location),
                is_config_selected(6),
            );
        } else {
            config_lines.push_row(
                metadata_line(
                    "Storage Backend",
                    app.app_config.effective_storage_backend(),
                ),
                is_config_selected(5),
            );
            config_lines.push_row(
                metadata_line("Storage Location", &active_storage_location),
                is_config_selected(6),
            );
        }
    }
    let config_paragraph = config_lines.focused(true).block(config_block);
    frame.render_widget(config_paragraph, area);
}

fn render_settings_config_file(app: &App, frame: &mut Frame, area: Rect, config_location: &str) {
    use crate::app::SettingsFocus;
    use crate::components::detail_view::{metadata_line, FieldSectionConfig};

    let config_file_focused = app.focus.settings_focus == SettingsFocus::ConfigFile;
    let config_file_section = FieldSectionConfig::new(" Config File ")
        .with_focus_indicator(" Config File [2] ")
        .focused(config_file_focused);
    let config_file_block = config_file_section.block();
    let is_cf_selected =
        |i: usize| config_file_focused && app.selection.settings_config_file.is_selected(i);
    let config_path_display = if config_location.is_empty() {
        "(unknown)".to_string()
    } else {
        config_location.to_string()
    };
    let config_exists =
        !config_location.is_empty() && std::path::Path::new(config_location).exists();
    let status = if config_exists { "Loaded" } else { "Not found" };
    let config_format = app.app_config.effective_configuration_format();
    let mut config_file_lines = RowList::new();
    config_file_lines.push_row(
        metadata_line("Path", &config_path_display),
        is_cf_selected(0),
    );
    config_file_lines.push_row(metadata_line("Status", status), is_cf_selected(1));
    config_file_lines.push_row(
        metadata_line("Configuration Format", config_format),
        is_cf_selected(2),
    );
    let config_file_paragraph = config_file_lines.focused(true).block(config_file_block);
    frame.render_widget(config_file_paragraph, area);
}

fn render_settings_storage(app: &App, frame: &mut Frame, area: Rect) {
    use crate::app::SettingsFocus;
    use crate::components::detail_view::{metadata_line, FieldSectionConfig};

    let storage_focused = app.focus.settings_focus == SettingsFocus::Storage;
    let storage_section = FieldSectionConfig::new(" Storage ")
        .with_focus_indicator(" Storage [3] ")
        .focused(storage_focused);
    let storage_block = storage_section.block();
    let is_storage_selected =
        |i: usize| storage_focused && app.selection.settings_storage.is_selected(i);
    let file_path = app.persistence.save_file.as_deref().unwrap_or("(none)");
    let backend = if app.persistence.save_file.is_some() {
        app.app_config.effective_storage_backend()
    } else {
        "(none)"
    };
    let instance_id = app.ctx.backend().instance_id().to_string();
    let mut storage_lines = RowList::new();
    storage_lines.push_row(metadata_line("File", file_path), is_storage_selected(0));
    storage_lines.push_row(metadata_line("Backend", backend), is_storage_selected(1));
    storage_lines.push_row(
        metadata_line("Instance ID", &instance_id),
        is_storage_selected(2),
    );
    storage_lines.push("");
    storage_lines.push_row(
        Line::from(vec![
            Span::styled("  [x] ", Style::default().fg(Color::Yellow)),
            Span::styled("Export Boards", Style::default().fg(Color::White)),
        ]),
        is_storage_selected(3),
    );
    let storage_paragraph = storage_lines.focused(true).block(storage_block);
    frame.render_widget(storage_paragraph, area);
}
