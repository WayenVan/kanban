use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Resize,
    Tick,
}

fn translate(event: CrosstermEvent) -> Option<Event> {
    match event {
        CrosstermEvent::Key(key) => {
            tracing::trace!(code = ?key.code, kind = ?key.kind, modifiers = ?key.modifiers, "raw key event");
            (key.kind == KeyEventKind::Press).then_some(Event::Key(key))
        }
        CrosstermEvent::Resize(..) => Some(Event::Resize),
        _ => None,
    }
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    shutdown_tx: mpsc::UnboundedSender<()>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(16)) => {
                        let mut had_event = false;
                        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                            let Some(event) = event::read().ok().and_then(translate) else {
                                continue;
                            };
                            had_event = true;
                            if tx.send(event).is_err() {
                                break;
                            }
                        }
                        if !had_event && tx.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Self { rx, shutdown_tx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub fn try_next(&mut self) -> Option<Event> {
        self.rx.try_recv().ok()
    }

    pub fn stop(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

pub fn should_quit(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventState, KeyModifiers};

    fn key(kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            kind,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_translate_resize_requests_a_redraw() {
        assert!(matches!(
            translate(CrosstermEvent::Resize(80, 24)),
            Some(Event::Resize)
        ));
    }

    #[test]
    fn test_translate_forwards_key_presses_only() {
        assert!(matches!(
            translate(CrosstermEvent::Key(key(KeyEventKind::Press))),
            Some(Event::Key(_))
        ));
        assert!(translate(CrosstermEvent::Key(key(KeyEventKind::Release))).is_none());
    }

    #[test]
    fn test_translate_ignores_unhandled_events() {
        assert!(translate(CrosstermEvent::FocusGained).is_none());
    }
}
