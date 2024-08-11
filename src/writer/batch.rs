use std::sync::Arc;

use crate::batch::{self, Batch};

use super::Writer;

pub struct BatchWriter {
    batch_store: Arc<dyn batch::Store + Sync + Send>,
    writer: Box<dyn Writer + Sync + Send>,
}

impl BatchWriter {
    pub fn new(
        batch_store: Arc<dyn batch::Store + Sync + Send>,
        writer: Box<dyn Writer + Sync + Send>,
    ) -> Self {
        Self {
            batch_store,
            writer,
        }
    }

    pub async fn write(&self) {
        for batch in self.batch_store.batches() {
            if self.is_ready(&batch) {
                self.writer.write(&batch).await;
                self.batch_store.delete_batch(batch.partition());
            }
        }
    }

    pub async fn flush(&self) {
        for batch in self.batch_store.batches() {
            self.writer.write(&batch).await;
        }
    }

    fn is_ready(&self, _batch: &Batch) -> bool {
        // Implement logic
        true
    }
}
