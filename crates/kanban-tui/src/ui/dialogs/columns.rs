use crate::app::App;
use crate::components::radio_list::ListItem as RadioItem;
use crate::components::*;
use crate::theme::*;
use ratatui::{style::Style, text::Line, Frame};

pub(crate) fn render_create_column_popup(app: &App, frame: &mut Frame) {
    let statuses = kanban_view::selection_dialog::DEFAULT_STATUS_POPUP_ORDER;
    let slots = [
        Slot::gap(1),
        Slot::line(5),
        Slot::line(9),
        Slot::gap(2),
        Slot::line(4),
        Slot::flexible(statuses.len() as u16, 1, 8),
        Slot::gap(0),
    ];
    let area = Popup::new("Create New Column")
        .border_style(focused_border())
        .content_height(slots_height(&slots))
        .render(frame);
    let rows = fit_rows(area, &slots);

    let name_focused = app.dialog_input.create_column_focus_is_name();

    if let Some(row) = rows[1] {
        frame.render_widget(field_label("Column Name:", name_focused), row);
    }
    if let Some(row) = rows[2] {
        render_input_field(
            frame,
            row,
            app.input.as_str(),
            app.input.cursor_display_col(),
            name_focused,
            Style::default(),
        );
    }

    if let Some(row) = rows[4] {
        frame.render_widget(
            field_label(
                "Default Status (Tab to switch, j/k to select):",
                !name_focused,
            ),
            row,
        );
    }
    if let Some(list_area) = rows[5] {
        let items: Vec<RadioItem<()>> = statuses
            .iter()
            .map(|(_, label)| RadioItem::selectable((), Line::from(format!("  {label}"))))
            .collect();
        RadioList::new(&items).render(
            frame,
            list_area,
            app.dialog_input.default_status_selection.get(),
        );
    }
}

pub(crate) fn render_rename_column_popup(app: &App, frame: &mut Frame) {
    render_input_popup(
        frame,
        "Rename Column",
        "New Column Name:",
        app.input.as_str(),
        app.input.cursor_display_col(),
    );
}

pub(crate) fn render_set_column_default_status_popup(app: &App, frame: &mut Frame) {
    use crate::components::{ColumnDefaultStatusDialog, SelectionDialog};
    let dialog = ColumnDefaultStatusDialog;
    dialog.render(app, frame);
}

pub(crate) fn render_delete_column_confirm_popup(_app: &App, frame: &mut Frame) {
    super::render_confirm_popup(
        frame,
        "Delete Column",
        "Are you sure you want to delete this column?\nAll cards will be moved to the first column."
            .to_string(),
    );
}

pub(crate) fn render_select_task_list_view_popup(app: &App, frame: &mut Frame) {
    use kanban_domain::TaskListView;

    let views = [
        TaskListView::Flat,
        TaskListView::GroupedByColumn,
        TaskListView::ColumnView,
    ];

    let selected = app.dialog_input.task_list_view_selection.get();

    let current_view = app.active_board().map(|board| board.task_list_view);

    let mut rows = RowList::new();
    for (idx, view) in views.iter().enumerate() {
        let is_current = current_view == Some(*view);
        let view_name = match view {
            TaskListView::Flat => {
                if is_current {
                    "Flat (current)"
                } else {
                    "Flat"
                }
            }
            TaskListView::GroupedByColumn => {
                if is_current {
                    "Grouped by Column (current)"
                } else {
                    "Grouped by Column"
                }
            }
            TaskListView::ColumnView => {
                if is_current {
                    "Column View (kanban board) (current)"
                } else {
                    "Column View (kanban board)"
                }
            }
        };
        rows.push_row(view_name, Some(idx) == selected);
    }

    render_list_popup(frame, "Select Task List View", None, rows, 50);
}
