use crate::app::App;
use crate::components::*;
use crate::ui::render_unavailable_panel;
use kanban_domain::LoadState;
use ratatui::{
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

pub(crate) fn render_create_card_popup(app: &App, frame: &mut Frame) {
    let Some(board) = app.active_board() else {
        render_input_popup(
            frame,
            "Create New Task",
            "Task Title:",
            app.input.as_str(),
            app.input.cursor_display_col(),
        );
        return;
    };

    let column_editable = app.dialog_input.create_card_column_is_editable();
    let sprint_visible = app.dialog_input.create_card_sprint_is_visible();

    if !column_editable && !sprint_visible {
        render_input_popup(
            frame,
            "Create New Task",
            "Task Title:",
            app.input.as_str(),
            app.input.cursor_display_col(),
        );
        return;
    }

    const PICKER_ROWS: u16 = 12;
    let mut slots = vec![
        Slot::gap(1),
        Slot::line(5),
        Slot::line(9),
        Slot::gap(2),
        Slot::line(4),
        Slot::line(8),
    ];
    if sprint_visible {
        slots.extend([
            Slot::gap(2),
            Slot::line(3),
            Slot::flexible(PICKER_ROWS, 3, 7),
        ]);
    }
    slots.push(Slot::gap(0));

    let area = Popup::new("Create New Task")
        .border_style(crate::theme::focused_border())
        .content_height(slots_height(&slots))
        .render(frame);
    let rows = fit_rows(area, &slots);

    let title_focused = app.dialog_input.create_card_focus_is_title();
    let column_focused = app.dialog_input.create_card_focus_is_column();
    let sprint_focused = app.dialog_input.create_card_focus_is_sprint();

    if let Some(row) = rows[1] {
        frame.render_widget(field_label("Task Title:", title_focused), row);
    }
    if let Some(row) = rows[2] {
        render_input_field(
            frame,
            row,
            app.input.as_str(),
            app.input.cursor_display_col(),
            title_focused,
            Style::default(),
        );
    }

    if let Some(row) = rows[4] {
        frame.render_widget(field_label("Column:", column_focused), row);
    }
    if let Some(row) = rows[5] {
        let column_text_style = if column_editable {
            crate::theme::normal_text()
        } else {
            crate::theme::label_text()
        };
        let column_input = &app.dialog_input.create_card_column_input;
        render_input_field(
            frame,
            row,
            column_input.as_str(),
            column_input.cursor_display_col(),
            column_focused,
            column_text_style,
        );
    }

    if !sprint_visible {
        return;
    }
    if let Some(row) = rows[7] {
        frame.render_widget(field_label("Sprint:", sprint_focused), row);
    }
    if let Some(picker_area) = rows[8] {
        let picker_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if sprint_focused {
                crate::theme::focused_border()
            } else {
                crate::theme::unfocused_border()
            });
        let picker_inner = picker_block.inner(picker_area);
        frame.render_widget(picker_block, picker_area);
        match app.model.board_sprints_state(board.id) {
            LoadState::Loaded(sprints) => {
                app.dialog_input.create_card_sprint_picker.render(
                    frame,
                    picker_inner,
                    sprints,
                    board,
                    chrono::Utc::now(),
                );
            }
            other => render_unavailable_panel(frame, picker_inner, "Sprint", &other),
        }
    }
}

pub(crate) fn render_rename_card_popup(app: &App, frame: &mut Frame) {
    render_input_popup(
        frame,
        "Rename Task",
        "New Task Title:",
        app.input.as_str(),
        app.input.cursor_display_col(),
    );
}

pub(crate) fn render_set_card_points_popup(app: &App, frame: &mut Frame) {
    render_input_popup(
        frame,
        "Set Points",
        "Points (1-5 or empty):",
        app.input.as_str(),
        app.input.cursor_display_col(),
    );
}

pub(crate) fn render_set_card_priority_popup(app: &App, frame: &mut Frame) {
    use crate::components::{PriorityDialog, SelectionDialog};
    let dialog = PriorityDialog;
    dialog.render(app, frame);
}

pub(crate) fn render_set_multiple_cards_priority_popup(app: &App, frame: &mut Frame) {
    use crate::components::{BulkPriorityDialog, SelectionDialog};
    let dialog = BulkPriorityDialog {
        count: app.multi_select.selected_cards.len(),
    };
    dialog.render(app, frame);
}

pub(crate) fn render_order_cards_popup(app: &App, frame: &mut Frame) {
    use crate::components::{SelectionDialog, SortFieldDialog};
    let dialog = SortFieldDialog;
    dialog.render(app, frame);
}

pub(crate) fn render_order_boards_popup(app: &App, frame: &mut Frame) {
    use crate::components::{BoardSortFieldDialog, SelectionDialog};
    let dialog = BoardSortFieldDialog;
    dialog.render(app, frame);
}

pub(crate) fn render_assign_sprint_popup(app: &App, frame: &mut Frame) {
    use crate::components::{SelectionDialog, SprintAssignDialog};
    let dialog = SprintAssignDialog;
    dialog.render(app, frame);
}

pub(crate) fn render_assign_multiple_cards_popup(app: &App, frame: &mut Frame) {
    let title = format!(
        "Assign {} Cards to Sprint",
        app.multi_select.selected_cards.len()
    );
    let Some(board) = app.active_board() else {
        list_popup_frame(frame, &title, Some("Select sprint:"), 1, 60);
        return;
    };
    let sprints = app.model.board_sprints_state(board.id);
    let list_rows = match &sprints {
        LoadState::Loaded(sprints) => {
            kanban_view::sprint_assign_list::build_entries(sprints, board.id, chrono::Utc::now())
                .len()
        }
        _ => 1,
    };
    let list = list_popup_frame(frame, &title, Some("Select sprint:"), list_rows as u16, 60);
    match sprints {
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
