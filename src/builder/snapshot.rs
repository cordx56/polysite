use super::metadata::Metadata;
use std::sync::{Arc, RwLock};
use tokio::{spawn, sync::Notify};

/// SnapshotStage manager
/// manages 1 compiler snapshot stage.
///
/// stage count starts from 0.
/// stage count means "running stage" number.
#[derive(Clone)]
pub struct SnapshotStage {
    notify: Arc<Notify>,
    metas: Arc<RwLock<Vec<Metadata>>>,
}
impl SnapshotStage {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            metas: Arc::new(RwLock::new(Vec::new())),
        }
    }
    /// Get current, waiting stage
    pub fn current_stage(&self) -> usize {
        self.metas.read().unwrap().len()
    }
    /// Clone self [`Notify`]
    pub fn get_notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }
    /// Notifies all waiters
    pub fn push(&self, meta: Metadata) {
        self.metas.write().unwrap().push(meta);
        self.notify.notify_waiters();
    }
    pub fn metas(&self) -> Arc<RwLock<Vec<Metadata>>> {
        self.metas.clone()
    }
}

/// SnapshotManager
/// manages 1 rule snapshots
#[derive(Clone)]
pub(crate) struct SnapshotManager {
    stages: Arc<RwLock<Vec<SnapshotStage>>>,
    notify: Arc<Notify>,
}
impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            stages: Arc::new(RwLock::new(Vec::new())),
            notify: Arc::new(Notify::new()),
        }
    }
    /// Register new [`SnapshotStage`]
    pub fn push(&self, stage: SnapshotStage) {
        let stage_notify = stage.get_notify();
        self.stages.write().unwrap().push(stage);
        let notify = self.notify.clone();
        spawn(async move {
            loop {
                stage_notify.notified().await;
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
                .stages
                .read()
                .unwrap()
                .iter()
                .filter(|s| s.metas().read().unwrap().len() < stage)
                .next()
                .is_none()
            {
                return;
            }
            self.notify.notified().await;
        }
    }
    pub fn current_stage(&self) -> Option<usize> {
        self.stages
            .read()
            .unwrap()
            .iter()
            .map(|s| s.current_stage())
            .min()
    }
    /// Get [`Metadata`]
    pub fn metadata(&self) -> Option<Vec<Metadata>> {
        if let Some(current) = self.current_stage() {
            if 0 < current {
                let res = self
                    .stages
                    .read()
                    .unwrap()
                    .iter()
                    .map(|s| s.metas().read().unwrap().get(current - 1).unwrap().clone())
                    .collect();
                return Some(res);
            }
        }
        None
    }
}
