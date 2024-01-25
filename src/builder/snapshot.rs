use std::sync::Arc;
use tokio::{
    spawn,
    sync::{Mutex, Notify},
};

/// SnapshotStage manager
/// manages 1 compiler snapshot stage.
///
/// stage count starts from 0.
/// stage count means "running stage" number.
#[derive(Clone)]
pub struct SnapshotStage {
    notify: Arc<Notify>,
    stage: Arc<Mutex<usize>>,
}
impl SnapshotStage {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            stage: Arc::new(Mutex::new(0)),
        }
    }
    /// Get current, waiting stage
    pub async fn current_stage(&self) -> usize {
        *self.stage.lock().await
    }
    /// Wait until notified and return next stage count
    pub async fn notified(&self) -> usize {
        self.notify.notified().await;
        *self.stage.lock().await
    }
    /// Notifies all waiters
    pub async fn notify_waiters(&self) {
        *self.stage.lock().await += 1;
        self.notify.notify_waiters();
    }
}

/// SnapshotManager
/// manages 1 rule snapshots
#[derive(Debug)]
pub(crate) struct SnapshotManager {
    current_stages: Arc<Mutex<Vec<usize>>>,
    notify: Arc<Notify>,
}
impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            current_stages: Arc::new(Mutex::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        }
    }
    /// Register new `SnapshotStage`
    pub async fn push(&self, stage: SnapshotStage) {
        let current_stage = stage.current_stage().await;
        let current_stages = self.current_stages.clone();
        let mut current_stages_locked = self.current_stages.lock().await;
        let index = current_stages_locked.len();
        current_stages_locked.push(current_stage);
        let notify = self.notify.clone();
        spawn(async move {
            loop {
                let next = stage.notified().await;
                *current_stages.lock().await.get_mut(index).unwrap() = next;
                notify.notify_waiters();
            }
        });
    }
    /// Wait until all snapshots' stages became specified stage
    ///
    /// Stage number is minimum required stage number.
    /// In most cases, you will specify stage number to 1.
    pub async fn wait_until(&self, stage: usize) {
        loop {
            if self
                .current_stages
                .lock()
                .await
                .iter()
                .filter(|s| **s < stage)
                .next()
                .is_some()
            {
                return;
            }
            self.notify.notified().await;
        }
    }
}
