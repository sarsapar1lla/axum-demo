use std::sync::Arc;

use tokio::{sync::broadcast::Sender, task::JoinHandle};

use crate::writer::BatchWriter;

pub async fn hook(
    shutdown_sender: Sender<bool>,
    batch_writer: Arc<BatchWriter>,
    handler_task: JoinHandle<()>,
    writer_task: JoinHandle<()>,
) {
    signal().await;

    tracing::info!("Sending shutdown signal");
    shutdown_sender.send(true).unwrap();

    tracing::info!("Awaiting shutdown of event handler task");
    handler_task.await.unwrap();

    tracing::info!("Awaiting shutdown of batch writer task");
    writer_task.await.unwrap();
    batch_writer.flush().await;
}

async fn signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.unwrap();
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
