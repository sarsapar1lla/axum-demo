use std::sync::Arc;

use crate::{deleter::MessageDeleter, processor::NotificationProcessor, supplier::Supplier};

pub struct EventHandler {
    supplier: Arc<dyn Supplier + Send + Sync>,
    processor: Arc<dyn NotificationProcessor + Send + Sync>,
    deleter: Arc<dyn MessageDeleter + Send + Sync>,
}

impl EventHandler {
    pub fn new(
        supplier: Arc<dyn Supplier + Send + Sync>,
        processor: Arc<dyn NotificationProcessor + Send + Sync>,
        deleter: Arc<dyn MessageDeleter + Send + Sync>,
    ) -> Self {
        Self {
            supplier,
            processor,
            deleter,
        }
    }

    pub async fn handle(&self) {
        let notifications = self.supplier.get().await;

        if notifications.is_empty() {
            return;
        }

        tracing::info!("Processing {} notifications", notifications.len());
        for notification in notifications {
            self.processor.process(&notification).await;
            tracing::info!("Deleting message '{}'", notification.message_id());
            self.deleter.delete(notification.receipt_handle()).await;
        }
    }
}
