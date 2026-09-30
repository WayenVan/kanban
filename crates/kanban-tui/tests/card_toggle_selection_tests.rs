use kanban_domain::{CreateCardOptions, KanbanOperations};
use kanban_tui::app::focus::Focus;
use kanban_tui::App;

fn app_with_cards() -> (App, Vec<uuid::Uuid>) {
    let mut app = App::test_default();
    let board = app.ctx.create_board("Board".to_string(), None).unwrap();
    let column = app
        .ctx
        .create_column(board.id, "Todo".to_string(), Some(0))
        .unwrap();
    let cards = ["one", "two", "three"]
        .iter()
        .map(|title| {
            app.ctx
                .create_card(
                    board.id,
                    column.id,
                    (*title).to_string(),
                    CreateCardOptions::default(),
                )
                .unwrap()
                .id
        })
        .collect();
    app.selection.active_board_id = Some(board.id);
    app.reload_model();
    app.prepare_frame();
    app.focus.active = Focus::Cards;
    (app, cards)
}

#[test]
fn test_toggle_current_card_selection_adds_and_removes_only_the_current_card() {
    let (mut app, _) = app_with_cards();

    app.handle_toggle_current_card_selection();
    assert_eq!(app.multi_select.selected_cards.len(), 1);
    assert!(!app.multi_select.selection_mode_active);

    // Moving does not extend the selection outside selection mode.
    app.handle_navigation_down();
    app.handle_toggle_current_card_selection();
    assert_eq!(app.multi_select.selected_cards.len(), 2);

    app.handle_toggle_current_card_selection();
    assert_eq!(app.multi_select.selected_cards.len(), 1);
}

#[test]
fn test_escape_clears_cards_picked_outside_selection_mode() {
    let (mut app, _) = app_with_cards();

    app.handle_toggle_current_card_selection();
    app.handle_escape_key();

    assert!(app.multi_select.selected_cards.is_empty());
    assert_eq!(app.focus.active, Focus::Cards);
}

#[test]
fn test_rename_card_prefills_the_title_and_saves_the_new_one() {
    let (mut app, cards) = app_with_cards();

    app.handle_rename_card_key();
    assert_eq!(app.input.as_str(), "one");

    app.input.set("  renamed  ".to_string());
    app.rename_card();

    let card = app
        .model
        .card_by_id_state(cards[0])
        .loaded()
        .copied()
        .unwrap();
    assert_eq!(card.title, "renamed");
}

#[test]
fn test_rename_card_ignores_a_blank_title() {
    let (mut app, cards) = app_with_cards();

    app.handle_rename_card_key();
    app.input.set("   ".to_string());
    app.rename_card();

    let card = app
        .model
        .card_by_id_state(cards[0])
        .loaded()
        .copied()
        .unwrap();
    assert_eq!(card.title, "one");
}
