mod helpers;

use kanban_domain::KanbanOperations;
use kanban_persistence::{PersistenceStore, StoreSnapshot};
use kanban_tui::app::mode::{AppMode, DialogMode};
use std::time::Duration;

async fn write_external_board(path: &str, name: &str) {
    let store = kanban_persistence_json::JsonFileStore::new(path);
    let (existing, _) = store.load().await.unwrap();
    let mut snapshot: kanban_domain::Snapshot = serde_json::from_slice(&existing.data).unwrap();
    snapshot
        .boards
        .push(kanban_domain::Board::new(name, None::<String>));
    store
        .save(StoreSnapshot {
            data: serde_json::to_vec(&snapshot).unwrap(),
            metadata: kanban_persistence::PersistenceMetadata::new(store.instance_id()),
        })
        .await
        .unwrap();
}

async fn board_names_on_disk(path: &str) -> Vec<String> {
    let store = kanban_persistence_json::JsonFileStore::new(path);
    let (snapshot, _) = store.load().await.unwrap();
    let snapshot: kanban_domain::Snapshot = serde_json::from_slice(&snapshot.data).unwrap();
    let mut names: Vec<String> = snapshot.boards.into_iter().map(|b| b.name).collect();
    names.sort();
    names
}

#[tokio::test(flavor = "multi_thread")]
async fn test_save_conflict_releases_pending_save_and_opens_conflict_dialog() {
    let dir = tempfile::tempdir().unwrap();
    let path = helpers::create_test_json_file(dir.path(), "source.json", &["Original"]).await;
    let (mut app, _save_rx) = kanban_tui::App::new(Some(path)).await.unwrap();
    app.load_initial_state().await;
    app.ctx.create_board("Local".to_string(), None).unwrap();
    assert!(app.ctx.save_coordinator.has_pending_saves());

    app.handle_save_conflict();

    assert!(
        !app.ctx.save_coordinator.has_pending_saves(),
        "a conflicted save is finished; leaving it pending hides every later external change"
    );
    assert!(app.ctx.has_conflict());
    assert!(app.ctx.is_dirty(), "the local edit is still unsaved");
    assert_eq!(app.mode, AppMode::Dialog(DialogMode::ConflictResolution));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_save_conflict_replaces_open_external_change_dialog() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = helpers::setup_app_with_json_file(dir.path()).await;
    app.open_dialog(DialogMode::ExternalChangeDetected);

    app.handle_save_conflict();

    assert_eq!(app.mode, AppMode::Dialog(DialogMode::ConflictResolution));
    app.handle_conflict_resolution_popup(crossterm::event::KeyCode::Esc);
    assert_ne!(
        app.mode,
        AppMode::Dialog(DialogMode::ExternalChangeDetected),
        "the superseded external-change dialog must not reappear underneath"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_save_worker_reports_conflict_to_app() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = helpers::setup_app_with_json_file_and_save_worker(dir.path()).await;
    let path = app.persistence.save_file.clone().unwrap();
    write_external_board(&path, "External").await;

    app.ctx.create_board("Local".to_string(), None).unwrap();

    let rx = app
        .persistence
        .save_conflict_rx
        .as_mut()
        .expect("spawn_save_worker wires a conflict channel");
    tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("the worker must report the conflict instead of swallowing it")
        .expect("conflict channel closed");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_force_overwrite_local_replaces_external_change_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = helpers::setup_app_with_json_file(dir.path()).await;
    let path = app.persistence.save_file.clone().unwrap();
    write_external_board(&path, "External").await;
    app.ctx.create_board("Local".to_string(), None).unwrap();
    assert!(app.ctx.save().await.unwrap_err().is_conflict_detected());
    app.handle_save_conflict();

    app.force_overwrite_local().await;

    assert_eq!(
        board_names_on_disk(&path).await,
        vec!["Local".to_string(), "OriginalBoard".to_string()],
        "keep-mine must write local state over the external change"
    );
    assert!(!app.ctx.has_conflict());
    assert!(!app.ctx.is_dirty());
    assert!(app.save_error.is_none());
}
