use std::sync::Arc;

use axum::async_trait;
use tokio::{sync::broadcast::Receiver, task::JoinHandle, time::Interval};

use crate::{handler::EventHandler, writer::BatchWriter};

#[async_trait]
pub trait Task {
    async fn run(&self);
}

#[async_trait]
impl Task for EventHandler {
    async fn run(&self) {
        self.handle().await;
    }
}

#[async_trait]
impl Task for BatchWriter {
    async fn run(&self) {
        self.write().await;
    }
}

pub fn task(
    task: Arc<dyn Task + Sync + Send>,
    mut interval: Interval,
    mut cancel: Receiver<()>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            task.run().await;
            tokio::select! {
                _ = cancel.recv() => { break; }
                _ = interval.tick() => { continue; }
            }
        }
    })
}
