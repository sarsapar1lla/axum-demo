use std::{sync::Arc, time::Duration};

use tokio::{sync::broadcast::Receiver, task::JoinHandle};

use crate::{handler::EventHandler, writer::BatchWriter};

pub fn handler(handler: EventHandler, mut shutdown: Receiver<bool>) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = shutdown.recv() => {}
            () = handle_events(handler) => {}
        }
    })
}

pub fn batch_writer(
    batch_writer: Arc<BatchWriter>,
    mut shutdown: Receiver<bool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        tokio::select! {
            _ = shutdown.recv() => {}
            () = write_batches(batch_writer) => {}
        }
    })
}

async fn handle_events(handler: EventHandler) {
    let mut interval = tokio::time::interval(Duration::from_millis(5_000));
    loop {
        interval.tick().await;
        handler.handle().await;
    }
}

async fn write_batches(batch_writer: Arc<BatchWriter>) {
    let mut interval = tokio::time::interval(Duration::from_millis(10_000));
    loop {
        interval.tick().await;
        tracing::info!("Writing on a schedule");
        batch_writer.write().await;
    }
}
