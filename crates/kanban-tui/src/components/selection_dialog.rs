use crate::app::App;
use crate::components::{list_popup_frame, render_list_popup, RowList};
use crate::ui::{load_state_body, render_unavailable_panel};
use kanban_domain::{LoadState, SprintStatus};
use kanban_view::selection_dialog::{
    popup_index_of_board_sort_field, popup_index_of_sort_field, BOARD_SORT_FIELD_POPUP_ORDER,
    SORT_FIELD_POPUP_ORDER,
};
use kanban_view::sprint_assign_list::build_entries;
use ratatui::Frame;

pub trait SelectionDialog {
    fn title(&self) -> &str;
    fn get_current_selection(&self, app: &App) -> usize;
    fn options_count(&self, app: &App) -> usize;
    fn render(&self, app: &App, frame: &mut Frame);
}

pub struct PriorityDialog;

impl SelectionDialog for PriorityDialog {
    fn title(&self) -> &str {
        "Set Priority"
    }

    fn get_current_selection(&self, app: &App) -> usize {
        app.get_current_priority_selection_index()
    }

    fn options_count(&self, _app: &App) -> usize {
        4 // Low, Medium, High, Critical
    }

    fn render(&self, app: &App, frame: &mut Frame) {
        let rows = priority_rows(app.dialog_input.priority_selection.get());

        render_list_popup(frame, "Set Priority", None, rows, 30);
    }
}

pub struct BulkPriorityDialog {
    pub count: usize,
}

impl SelectionDialog for BulkPriorityDialog {
    fn title(&self) -> &str {
        "Set Priority (Bulk)"
    }

    fn get_current_selection(&self, _app: &App) -> usize {
        0
    }

    fn options_count(&self, _app: &App) -> usize {
        4 // Low, Medium, High, Critical
    }

    fn render(&self, app: &App, frame: &mut Frame) {
        let rows = priority_rows(app.dialog_input.priority_selection.get());

        let title = format!("Set Priority ({} cards)", self.count);
        render_list_popup(frame, &title, None, rows, 35);
    }
}

pub struct SortFieldDialog;

impl SelectionDialog for SortFieldDialog {
    fn title(&self) -> &str {
        "Order Tasks By"
    }

    fn get_current_selection(&self, app: &App) -> usize {
        app.get_current_sort_field_selection_index()
    }

    fn options_count(&self, _app: &App) -> usize {
        SORT_FIELD_POPUP_ORDER.len()
    }

    fn render(&self, app: &App, frame: &mut Frame) {
        use crate::components::render_selection_popup_with_lines;
        use kanban_domain::SortOrder;

        let active_idx = app.filter.current_sort_field.map(popup_index_of_sort_field);

        render_selection_popup_with_lines(
            frame,
            "Order Tasks By",
            Some("Select sort field:"),
            SORT_FIELD_POPUP_ORDER.iter(),
            |_idx, entry, _is_selected, is_active| {
                let (_field, label) = **entry;
                let order_indicator = if is_active {
                    match app.filter.current_sort_order {
                        Some(SortOrder::Ascending) => Some(" (↑)".to_string()),
                        Some(SortOrder::Descending) => Some(" (↓)".to_string()),
                        None => None,
                    }
                } else {
                    None
                };

                (label.to_string(), order_indicator)
            },
            app.filter.sort_field_selection.get(),
            active_idx,
            60,
        );
    }
}

/// Field picker for the PROJECTS panel sort — the board-side analogue of
/// [`SortFieldDialog`]. Same list-with-active-order-indicator layout, but
/// backed by [`BOARD_SORT_FIELD_POPUP_ORDER`] and whichever partition (live or
/// archived) is currently active on the model.
pub struct BoardSortFieldDialog;

impl SelectionDialog for BoardSortFieldDialog {
    fn title(&self) -> &str {
        "Order Projects By"
    }

    fn get_current_selection(&self, app: &App) -> usize {
        app.get_current_board_sort_field_selection_index()
    }

    fn options_count(&self, _app: &App) -> usize {
        BOARD_SORT_FIELD_POPUP_ORDER.len()
    }

    fn render(&self, app: &App, frame: &mut Frame) {
        use crate::components::render_selection_popup_with_lines;
        use kanban_domain::SortOrder;

        let want_archived = matches!(app.get_base_mode(), crate::app::AppMode::ArchivedBoardsView);
        let (active_field, active_order) = app.controller.board_sort(want_archived);
        let active_idx = Some(popup_index_of_board_sort_field(active_field));

        render_selection_popup_with_lines(
            frame,
            "Order Projects By",
            Some("Select sort field:"),
            BOARD_SORT_FIELD_POPUP_ORDER.iter(),
            |_idx, entry, _is_selected, is_active| {
                let (_field, label) = **entry;
                let order_indicator = if is_active {
                    match active_order {
                        SortOrder::Ascending => Some(" (↑)".to_string()),
                        SortOrder::Descending => Some(" (↓)".to_string()),
                    }
                } else {
                    None
                };

                (label.to_string(), order_indicator)
            },
            app.filter.board_sort_field_selection.get(),
            active_idx,
            60,
        );
    }
}

pub struct CarryOverSprintDialog {
    pub card_count: usize,
}

impl SelectionDialog for CarryOverSprintDialog {
    fn title(&self) -> &str {
        "Carry Over to Sprint"
    }

    fn get_current_selection(&self, app: &App) -> usize {
        app.dialog_input
            .carry_over_sprint_selection
            .get()
            .unwrap_or(0)
    }

    fn options_count(&self, app: &App) -> usize {
        let Some(board) = app.active_board() else {
            return 0;
        };
        match app.model.board_sprints_state(board.id) {
            LoadState::Loaded(sprints) => sprints
                .iter()
                .filter(|s| s.status == SprintStatus::Planning)
                .count(),
            _ => 0,
        }
    }

    fn render(&self, app: &App, frame: &mut Frame) {
        use ratatui::style::{Color, Style};
        use ratatui::text::Span;

        let title = format!("Carry Over to Sprint ({} cards)", self.card_count);
        let mut lines = RowList::new();

        if let Some(board) = app.active_board() {
            match app.model.board_sprints_state(board.id) {
                LoadState::Loaded(sprints) => {
                    let planning_sprints: Vec<_> = sprints
                        .iter()
                        .filter(|s| s.status == SprintStatus::Planning)
                        .collect();

                    for (idx, sprint) in planning_sprints.iter().enumerate() {
                        let is_selected =
                            app.dialog_input.carry_over_sprint_selection.get() == Some(idx);

                        let prefix = if is_selected { "> " } else { "  " };
                        let sprint_name = sprint.formatted_name(board, None);

                        lines.push_row(
                            Span::styled(
                                format!("{}{}", prefix, sprint_name),
                                Style::default().fg(Color::White),
                            ),
                            is_selected,
                        );
                    }
                }
                other => lines.extend(load_state_body("Sprints", &other)),
            }
        }

        render_list_popup(frame, &title, Some("Select target sprint:"), lines, 60);
    }
}

pub struct SprintAssignDialog;

impl SelectionDialog for SprintAssignDialog {
    fn title(&self) -> &str {
        "Assign to Sprint"
    }

    fn get_current_selection(&self, app: &App) -> usize {
        app.get_current_sprint_selection_index()
    }

    fn options_count(&self, app: &App) -> usize {
        let Some(board) = app.active_board() else {
            return 1;
        };
        match app.model.board_sprints_state(board.id) {
            LoadState::Loaded(sprints) => {
                build_entries(sprints, board.id, chrono::Utc::now()).len()
            }
            _ => 1,
        }
    }

    fn render(&self, app: &App, frame: &mut Frame) {
        let list = list_popup_frame(
            frame,
            "Assign to Sprint",
            Some("Select sprint:"),
            self.options_count(app) as u16,
            60,
        );

        let Some(board) = app.active_board() else {
            return;
        };
        match app.model.board_sprints_state(board.id) {
            LoadState::Loaded(sprints) => {
                app.dialog_input.assign_sprint_picker.render(
                    frame,
                    list,
                    sprints,
                    board,
                    chrono::Utc::now(),
                );
            }
            other => render_unavailable_panel(frame, list, "Sprint", &other),
        }
    }
}

pub struct ColumnDefaultStatusDialog;

impl SelectionDialog for ColumnDefaultStatusDialog {
    fn title(&self) -> &str {
        "Set Default Status"
    }

    fn get_current_selection(&self, app: &App) -> usize {
        app.dialog_input.default_status_selection.get().unwrap_or(0)
    }

    fn options_count(&self, _app: &App) -> usize {
        kanban_view::selection_dialog::DEFAULT_STATUS_POPUP_ORDER.len()
    }

    fn render(&self, app: &App, frame: &mut Frame) {
        let selected = app.dialog_input.default_status_selection.get();
        let mut rows = RowList::new();
        for (idx, (_, label)) in kanban_view::selection_dialog::DEFAULT_STATUS_POPUP_ORDER
            .iter()
            .enumerate()
        {
            rows.push_row(*label, Some(idx) == selected);
        }

        render_list_popup(frame, "Set Default Status", None, rows, 30);
    }
}

fn priority_rows(selected: Option<usize>) -> RowList<'static> {
    use kanban_domain::CardPriority;

    let mut rows = RowList::new();
    for (idx, priority) in [
        CardPriority::Low,
        CardPriority::Medium,
        CardPriority::High,
        CardPriority::Critical,
    ]
    .iter()
    .enumerate()
    {
        rows.push_row(format!("{:?}", priority), Some(idx) == selected);
    }
    rows
}
