use super::{App, AppMode, DialogMode};

impl App {
    /// The save worker hit a file changed by another writer. The save is
    /// over (it will not be retried on its own), so release it and let the
    /// user choose between keeping local state and taking the file's.
    pub fn handle_save_conflict(&mut self) {
        self.ctx.save_coordinator.save_completed();
        self.ctx.set_conflict();
        self.needs_redraw = true;
        if self.mode == AppMode::Dialog(DialogMode::ExternalChangeDetected) {
            self.pop_mode();
        }
        if self.mode != AppMode::Dialog(DialogMode::ConflictResolution) {
            self.open_dialog(DialogMode::ConflictResolution);
        }
    }

    pub async fn force_overwrite_local(&mut self) {
        self.needs_redraw = true;
        self.ctx.clear_conflict();
        match self.ctx.force_save().await {
            Ok(()) => {
                self.clear_save_error();
                if !self.ctx.save_coordinator.has_pending_saves() {
                    self.ctx.mark_clean();
                }
            }
            Err(e) => {
                tracing::error!("Failed to force overwrite: {}", e);
                self.set_save_error(e.to_string());
            }
        }
    }
}
