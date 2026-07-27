use crate::tui::app::App;
use std::path::Path;

struct RestoreGuard<F: FnOnce()> {
    restore: Option<F>,
}

impl<F: FnOnce()> RestoreGuard<F> {
    fn new(restore: F) -> Self {
        Self {
            restore: Some(restore),
        }
    }
}

impl<F: FnOnce()> Drop for RestoreGuard<F> {
    fn drop(&mut self) {
        if let Some(restore) = self.restore.take() {
            restore();
        }
    }
}

pub fn run_tui(notes_dir: &Path) -> color_eyre::Result<()> {
    color_eyre::install()?;
    let _restore_guard = RestoreGuard::new(ratatui::restore);
    let terminal = ratatui::init();
    App::new(notes_dir).run(terminal)
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::RestoreGuard;

    #[test]
    fn restore_guard_runs_cleanup_when_dropped() {
        let restored = Arc::new(AtomicBool::new(false));
        let restored_by_guard = Arc::clone(&restored);

        {
            let _guard = RestoreGuard::new(move || {
                restored_by_guard.store(true, Ordering::SeqCst);
            });
        }

        assert!(restored.load(Ordering::SeqCst));
    }

    #[test]
    fn restore_guard_runs_cleanup_during_unwind() {
        let restored = Arc::new(AtomicBool::new(false));
        let restored_by_guard = Arc::clone(&restored);

        let unwind_result = catch_unwind(AssertUnwindSafe(move || {
            let _guard = RestoreGuard::new(move || {
                restored_by_guard.store(true, Ordering::SeqCst);
            });
            panic!("simulated TUI failure");
        }));

        assert!(unwind_result.is_err());
        assert!(restored.load(Ordering::SeqCst));
    }
}
