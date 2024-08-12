use std::{collections::HashMap, sync::Arc};

use axum::async_trait;
use chrono::{DateTime, Utc};
pub use extractor::EventExtractor;

use crate::{
    batch,
    model::{Event, Notification},
};

mod extractor;
mod transform;

#[async_trait]
pub trait NotificationProcessor {
    async fn process(&self, notification: &Notification);
}

pub struct NotificationProcessorImpl {
    extractor: Box<dyn EventExtractor + Sync + Send>,
    batch_store: Arc<dyn batch::Store + Sync + Send>,
}

impl NotificationProcessorImpl {
    pub fn new(
        extractor: Box<dyn EventExtractor + Sync + Send>,
        batch_store: Arc<dyn batch::Store + Sync + Send>,
    ) -> Self {
        Self {
            extractor,
            batch_store,
        }
    }
}

#[async_trait]
impl NotificationProcessor for NotificationProcessorImpl {
    async fn process(&self, notification: &Notification) {
        let bytes = self.extractor.extract(notification).await;
        let event = NotificationProcessorImpl::deserialise(&bytes);
        let flattened = transform::apply(&event, notification);
        let json = NotificationProcessorImpl::serialise(&flattened);
        let entry = NotificationProcessorImpl::entry(
            event.request().source(),
            notification.created(),
            &json,
        );
        self.batch_store.add(entry);
    }
}

impl NotificationProcessorImpl {
    fn deserialise(bytes: &[u8]) -> Event {
        serde_json::from_slice(bytes).unwrap()
    }

    fn serialise(flattened: &HashMap<String, String>) -> String {
        serde_json::to_string(flattened).unwrap()
    }

    fn entry(source: &str, created: &DateTime<Utc>, json: &str) -> batch::Entry {
        let partition = batch::Partition::new(source, created.date_naive());
        batch::Entry::new(partition, created, json)
    }
}
