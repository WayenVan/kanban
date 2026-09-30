use kanban_domain::{KanbanOperations, KanbanResult, UndoOperations};
use kanban_service::{AppConfig, KanbanContext, StoreManager};
use kanban_tui::tui_context::TuiContext;
use tempfile::TempDir;

fn test_store_manager() -> StoreManager {
    let mut registry = kanban_persistence::StoreRegistry::new();
    let mut backends = kanban_backend::KanbanBackendRegistry::new();
    backends.register(Box::new(kanban_persistence_sqlite::SqliteBackendFactory));
    registry.register(Box::new(kanban_persistence_json::JsonStoreFactory));
    backends.register(Box::new(kanban_persistence_json::JsonBackendFactory));
    StoreManager::new(registry, backends)
}

async fn open_tui_context(dir: &TempDir, file: &str) -> KanbanResult<TuiContext> {
    let path = dir.path().join(file);
    let locator = path.to_str().unwrap();
    let mut config = AppConfig::default();
    let sm = test_store_manager();
    sm.sync_backend_with_file(locator, &mut config);
    let backend = sm.make_backend(locator, &config).await?;
    let ctx = KanbanContext::open(backend, config).await?;
    let (tui_ctx, _, _) = TuiContext::new(ctx)?;
    Ok(tui_ctx)
}

// multi_thread: sqlx connection pool spawns background tasks that deadlock on single-threaded runtime
#[tokio::test(flavor = "multi_thread")]
async fn test_sqlite_mutation_leaves_context_clean() -> KanbanResult<()> {
    let dir = TempDir::new().unwrap();
    let mut tui_ctx = open_tui_context(&dir, "test.sqlite3").await?;

    tui_ctx.create_board("B".to_string(), None)?;

    assert!(
        !tui_ctx.is_dirty(),
        "SQLite commits each mutation synchronously; a lingering dirty flag turns \
         the backend's own WAL checkpoint into a spurious external-change popup"
    );
    Ok(())
}

// multi_thread: sqlx connection pool spawns background tasks that deadlock on single-threaded runtime
#[tokio::test(flavor = "multi_thread")]
async fn test_sqlite_undo_and_redo_leave_context_clean() -> KanbanResult<()> {
    let dir = TempDir::new().unwrap();
    let mut tui_ctx = open_tui_context(&dir, "test.sqlite3").await?;
    tui_ctx.create_board("B".to_string(), None)?;

    assert!(tui_ctx.undo()?.is_some());
    assert!(!tui_ctx.is_dirty(), "undo must not leave the context dirty");

    assert!(tui_ctx.redo()?.is_some());
    assert!(!tui_ctx.is_dirty(), "redo must not leave the context dirty");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_json_mutation_stays_dirty_until_save_completes() -> KanbanResult<()> {
    let dir = TempDir::new().unwrap();
    let mut tui_ctx = open_tui_context(&dir, "test.json").await?;

    tui_ctx.create_board("B".to_string(), None)?;

    assert!(
        tui_ctx.is_dirty(),
        "JSON writes go through the save worker; dirty must hold until it reports completion"
    );
    Ok(())
}
