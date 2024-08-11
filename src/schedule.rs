use std::{sync::Arc, time::Duration};

use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

use crate::{handler::EventHandler, writer::BatchWriter};

pub fn schedule_handler(handler: EventHandler, token: CancellationToken) {
    tokio::spawn(async move {
        tokio::select! {
            _ = token.cancelled() => {}
            _ = handle_events(handler) => {}
        }
    });
}

pub fn schedule_batch_writer(batch_writer: Arc<BatchWriter>, token: CancellationToken) {
    tokio::spawn(async move {
        tokio::select! {
            _ = token.cancelled() => {
                batch_writer.flush().await;
            }
            _ = write_batches(batch_writer.clone()) => {}
        }
    });
}

async fn handle_events(handler: EventHandler) {
    loop {
        handler.handle().await;
        tokio::time::sleep_until(Instant::now() + Duration::from_millis(5000)).await;
    }
}

async fn write_batches(batch_writer: Arc<BatchWriter>) {
    loop {
        batch_writer.write().await;
        tokio::time::sleep_until(Instant::now() + Duration::from_millis(10_000)).await;
    }
}
