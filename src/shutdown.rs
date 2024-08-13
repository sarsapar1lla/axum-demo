use std::sync::Arc;

use tokio::{sync::broadcast::Sender, task::JoinHandle};

use crate::writer::BatchWriter;

pub async fn hook(
    shutdown_sender: Sender<()>,
    batch_writer: Arc<BatchWriter>,
    background_tasks: Vec<JoinHandle<()>>,
) {
    signal().await;

    tracing::info!("Received shutdown signal. Notifying background tasks");
    shutdown_sender.send(()).unwrap();

    tracing::info!("Awaiting end of background tasks");
    for task in background_tasks {
        task.await.unwrap();
    }
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
