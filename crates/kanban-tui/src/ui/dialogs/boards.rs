use crate::app::App;
use crate::components::*;
use crate::theme::*;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub(crate) fn render_export_boards_popup(app: &App, frame: &mut Frame) {
    use crate::app::{ExportFormat, ExportStep};

    let Some(ref dialog) = app.export_dialog else {
        return;
    };

    match dialog.step {
        ExportStep::SelectBoards => {
            let slots = [
                Slot::gap(1),
                Slot::flexible(dialog.board_ids.len().max(1) as u16, 1, 9),
                Slot::gap(2),
                Slot::line(5),
                Slot::gap(0),
            ];
            let area = Popup::new("Select Boards to Export")
                .border_style(focused_border())
                .content_height(slots_height(&slots))
                .render(frame);
            let rows = fit_rows(area, &slots);

            let mut list = RowList::new();
            for (i, &id) in dialog.board_ids.iter().enumerate() {
                let name = app
                    .model
                    .board_by_id_state(id)
                    .loaded()
                    .copied()
                    .map(|b| b.name.as_str())
                    .unwrap_or("?");
                let checkbox = if dialog.board_selections.get(i).copied().unwrap_or(false) {
                    "[x] "
                } else {
                    "[ ] "
                };
                list.push_row(
                    Span::styled(
                        format!("{}{}", checkbox, name),
                        Style::default().fg(Color::White),
                    ),
                    i == dialog.cursor,
                );
            }

            if let Some(row) = rows[1] {
                let scroll =
                    scroll_offset_to_show(dialog.cursor, list.len(), usize::from(row.height));
                frame.render_widget(list.focused(true).scroll(scroll as u16), row);
            }
            if let Some(row) = rows[3] {
                let hint = Paragraph::new("Space: toggle | a: all | Enter: next | Esc: cancel")
                    .style(label_text());
                frame.render_widget(hint, row);
            }
        }
        ExportStep::ExportOptions => {
            let slots = [
                Slot::gap(1),
                Slot::line(9),
                Slot::gap(2),
                Slot::line(8),
                Slot::gap(3),
                Slot::line(5),
                Slot::gap(0),
            ];
            let area = Popup::new("Export Options")
                .border_style(focused_border())
                .content_height(slots_height(&slots))
                .render(frame);
            let rows = fit_rows(area, &slots);
            let draw =
                |frame: &mut Frame, widget: Paragraph, row: Option<ratatui::layout::Rect>| {
                    if let Some(row) = row {
                        frame.render_widget(widget, row);
                    }
                };

            let filename_label = Paragraph::new(Line::from(vec![
                Span::styled("Filename: ", Style::default().fg(Color::Cyan)),
                Span::styled(&dialog.filename, Style::default().fg(Color::White)),
                Span::styled("_", Style::default().fg(Color::Yellow)),
            ]));
            draw(frame, filename_label, rows[1]);

            let json_style = if dialog.format == ExportFormat::Json {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let sqlite_style = if dialog.format == ExportFormat::Sqlite {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let json_radio = if dialog.format == ExportFormat::Json {
                "(*)"
            } else {
                "( )"
            };
            let sqlite_radio = if dialog.format == ExportFormat::Sqlite {
                "(*)"
            } else {
                "( )"
            };

            let format_line = Paragraph::new(Line::from(vec![
                Span::styled("Format: ", Style::default().fg(Color::Cyan)),
                Span::styled(format!("{} JSON  ", json_radio), json_style),
                Span::styled(format!("{} SQLite", sqlite_radio), sqlite_style),
            ]));
            draw(frame, format_line, rows[3]);

            let hint = Paragraph::new(Line::from(vec![Span::styled(
                "Tab: format | Enter: export | Esc: back",
                Style::default().fg(Color::DarkGray),
            )]));
            draw(frame, hint, rows[5]);
        }
    }
}

pub(crate) fn render_delete_board_confirm_popup(app: &App, frame: &mut Frame) {
    // Counts are snapshotted when the dialog opens (handle_delete_board_key),
    // so the modal never re-scans the model per frame. This confirm ARCHIVES the
    // board (soft): its subtree stays in place and it moves to the archived-boards
    // view (`D`), where it can be restored or permanently deleted.
    let body = match app.dialog_input.board_delete_counts {
        Some(counts) if !counts.is_empty() => format!(
            "Archive this project and everything in it?\n\
             {} column(s), {} task(s), {} archived task(s), {} sprint(s)\n\
             It moves to the archived view (`D`); undo with `u`.",
            counts.columns, counts.cards, counts.archived, counts.sprints,
        ),
        _ => "This project is empty.\nArchive it?".to_string(),
    };
    super::render_confirm_popup(frame, "Archive Project", body);
}

pub(crate) fn render_delete_permanent_board_confirm_popup(app: &App, frame: &mut Frame) {
    let body = match app.dialog_input.board_delete_counts {
        Some(counts) if !counts.is_empty() => format!(
            "Permanently delete this project and everything in it?\n\
             {} column(s), {} task(s), {} archived task(s), {} sprint(s)\n\
             This CANNOT be undone from here.",
            counts.columns, counts.cards, counts.archived, counts.sprints,
        ),
        _ => "This project is empty.\nPermanently delete it?".to_string(),
    };
    super::render_confirm_popup(frame, "Permanently Delete Project", body);
}

pub(crate) fn render_create_board_popup(app: &App, frame: &mut Frame) {
    render_input_popup(
        frame,
        "Create New Project",
        "Project Name:",
        app.input.as_str(),
        app.input.cursor_display_col(),
    );
}

pub(crate) fn render_rename_board_popup(app: &App, frame: &mut Frame) {
    render_input_popup(
        frame,
        "Rename Project",
        "New Project Name:",
        app.input.as_str(),
        app.input.cursor_display_col(),
    );
}

pub(crate) fn render_export_board_popup(app: &App, frame: &mut Frame) {
    render_input_popup(
        frame,
        "Export Project",
        "Filename:",
        app.input.as_str(),
        app.input.cursor_display_col(),
    );
}

pub(crate) fn render_export_all_popup(app: &App, frame: &mut Frame) {
    render_input_popup(
        frame,
        "Export All Projects",
        "Filename:",
        app.input.as_str(),
        app.input.cursor_display_col(),
    );
}

pub(crate) fn render_import_board_popup(app: &App, frame: &mut Frame) {
    let files = &app.dialog_input.import_files;
    if files.is_empty() {
        let list = list_popup_frame(
            frame,
            "Import Projects",
            Some("Select a JSON file to import:"),
            1,
            60,
        );
        let empty_msg =
            Paragraph::new("No JSON files found in current directory").style(label_text());
        frame.render_widget(empty_msg, list);
        return;
    }
    let mut rows = RowList::new();
    for (idx, filename) in files.iter().enumerate() {
        rows.push_row(
            styled_list_item(filename, &ListItemConfig::new()),
            app.dialog_input.import_selection.get() == Some(idx),
        );
    }
    render_list_popup(
        frame,
        "Import Projects",
        Some("Select a JSON file to import:"),
        rows,
        60,
    );
}

pub(crate) fn render_set_branch_prefix_popup(app: &App, frame: &mut Frame) {
    render_input_popup(
        frame,
        "Set Branch Prefix",
        "Branch Prefix:",
        app.input.as_str(),
        app.input.cursor_display_col(),
    );
}

pub(crate) fn render_choose_storage_file_popup(app: &App, frame: &mut Frame) {
    use crate::app::StorageBackendChoice;

    let slots = [
        Slot::gap(0),
        Slot::flexible(3, 1, 6), // description
        Slot::gap(1),
        Slot::line(5),           // "Filename:" label
        Slot::line(9),           // input
        Slot::flexible(2, 1, 4), // resolved-path preview (wraps if long)
        Slot::gap(2),
        Slot::line(7), // format radio
        Slot::gap(1),
        Slot::line(3), // hint
        Slot::gap(0),
    ];
    let area = Popup::new("No board file configured")
        .width_percent(70)
        .content_height(slots_height(&slots))
        .render(frame);
    let rows = fit_rows(area, &slots);
    let draw = |frame: &mut Frame, widget: Paragraph, row: Option<ratatui::layout::Rect>| {
        if let Some(row) = row {
            frame.render_widget(widget, row);
        }
    };

    let bold_normal = normal_text().add_modifier(Modifier::BOLD);

    let description = vec![
        Line::from(Span::styled(
            "Enter a filename to create a board file, or press Escape",
            normal_text(),
        )),
        Line::from(Span::styled(
            "to continue without one - you can export it at",
            normal_text(),
        )),
        Line::from(vec![
            Span::styled("any time with '", normal_text()),
            Span::styled("x", bold_normal),
            Span::styled("'.", normal_text()),
        ]),
    ];
    draw(
        frame,
        Paragraph::new(description).wrap(ratatui::widgets::Wrap { trim: false }),
        rows[1],
    );

    draw(frame, field_label("Filename:", true), rows[3]);
    if let Some(row) = rows[4] {
        render_input_field(
            frame,
            row,
            app.input.as_str(),
            app.input.cursor_display_col(),
            true,
            Style::default(),
        );
    }

    let resolved = display_dialog_path(app.input.as_str());
    let preview = Paragraph::new(Line::from(vec![
        Span::styled("Will be saved at: ", label_text()),
        Span::styled(resolved, normal_text()),
    ]))
    .wrap(ratatui::widgets::Wrap { trim: false });
    draw(frame, preview, rows[5]);

    let radio = Line::from(vec![
        Span::styled("Format: ", highlight_text()),
        radio_marker(
            app.choose_storage_backend,
            StorageBackendChoice::Json,
            "JSON",
        ),
        Span::styled("   ", normal_text()),
        radio_marker(
            app.choose_storage_backend,
            StorageBackendChoice::Sqlite,
            "SQLite",
        ),
        Span::styled("    (", normal_text()),
        Span::styled("Tab", bold_normal),
        Span::styled(" to toggle)", normal_text()),
    ]);
    draw(frame, Paragraph::new(radio), rows[7]);

    let hint = Line::from(vec![
        Span::styled("Enter", bold_normal),
        Span::styled(" — create file   ", normal_text()),
        Span::styled("Esc", bold_normal),
        Span::styled(" — continue in memory", normal_text()),
    ]);
    draw(frame, Paragraph::new(hint), rows[9]);
}

fn radio_marker(
    selected: crate::app::StorageBackendChoice,
    choice: crate::app::StorageBackendChoice,
    label: &str,
) -> Span<'static> {
    let marker = if selected == choice { "(*)" } else { "( )" };
    let style = if selected == choice {
        highlight_text()
    } else {
        normal_text()
    };
    Span::styled(format!("{} {}", marker, label), style)
}

fn resolve_dialog_path(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let path = std::path::Path::new(input);
    if path.is_absolute() {
        return path.display().to_string();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path).display().to_string())
        .unwrap_or_else(|_| input.to_string())
}

/// Resolves `input` to an absolute path, then substitutes `$HOME` with `~`
/// when the resolved path lies under the user's home directory. On Windows
/// or when `HOME` is unset, returns the absolute path unchanged.
fn display_dialog_path(input: &str) -> String {
    shrink_home(&resolve_dialog_path(input))
}

fn shrink_home(abs: &str) -> String {
    if abs.is_empty() {
        return String::new();
    }
    let Some(home_os) = std::env::var_os("HOME") else {
        return abs.to_string();
    };
    let home = home_os.to_string_lossy();
    if home.is_empty() {
        return abs.to_string();
    }
    if abs == home.as_ref() {
        return "~".to_string();
    }
    if let Some(rest) = abs.strip_prefix(home.as_ref()) {
        if rest.starts_with('/') {
            return format!("~{}", rest);
        }
    }
    abs.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_home<R>(home: Option<&str>, f: impl FnOnce() -> R) -> R {
        let _g = crate::test_helpers::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let prev = std::env::var_os("HOME");
        match home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
        let r = f();
        match prev {
            Some(p) => std::env::set_var("HOME", p),
            None => std::env::remove_var("HOME"),
        }
        r
    }

    #[test]
    fn test_shrink_home_substitutes_home_prefix() {
        with_home(Some("/home/max"), || {
            assert_eq!(
                shrink_home("/home/max/foo/boards.json"),
                "~/foo/boards.json"
            );
        });
    }

    #[test]
    fn test_shrink_home_returns_tilde_for_exact_home() {
        with_home(Some("/home/max"), || {
            assert_eq!(shrink_home("/home/max"), "~");
        });
    }

    #[test]
    fn test_shrink_home_leaves_path_outside_home_unchanged() {
        with_home(Some("/home/max"), || {
            assert_eq!(shrink_home("/var/log/x.json"), "/var/log/x.json");
        });
    }

    #[test]
    fn test_shrink_home_ignores_partial_prefix_match() {
        with_home(Some("/home/max"), || {
            // "/home/maximus/x" must not become "imus/x" — only true segment
            // boundaries count.
            assert_eq!(shrink_home("/home/maximus/x"), "/home/maximus/x");
        });
    }

    #[test]
    fn test_shrink_home_returns_input_when_home_unset() {
        with_home(None, || {
            assert_eq!(shrink_home("/some/path"), "/some/path");
        });
    }

    #[test]
    fn test_shrink_home_returns_empty_for_empty_input() {
        with_home(Some("/home/max"), || {
            assert_eq!(shrink_home(""), "");
        });
    }

    // POSIX-only: relies on `/home/max/...` being absolute and on the `HOME`
    // env var. `display_dialog_path` documents that it returns the resolved
    // path unchanged on Windows, where forward-slash paths aren't absolute
    // and there is no `HOME` variable to substitute.
    #[cfg(unix)]
    #[test]
    fn test_display_dialog_path_shrinks_absolute_input_under_home() {
        with_home(Some("/home/max"), || {
            assert_eq!(
                display_dialog_path("/home/max/notes/boards.json"),
                "~/notes/boards.json"
            );
        });
    }
}
