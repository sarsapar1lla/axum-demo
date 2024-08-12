use std::sync::{Arc, LazyLock};

use chrono::{TimeDelta, Utc};

use crate::batch::{self, Batch};

use super::Writer;

const MAX_BATCH_SIZE: usize = 1;
static MAX_BATCH_AGE: LazyLock<TimeDelta> = LazyLock::new(|| TimeDelta::minutes(60));

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
            if BatchWriter::is_ready(&batch) {
                self.writer.write(&batch).await;
                self.batch_store.delete_batch(batch.partition());
            }
        }
    }

    pub async fn flush(&self) {
        tracing::info!("Writing all batches prior to shutdown...");
        for batch in self.batch_store.batches() {
            self.writer.write(&batch).await;
        }
    }

    fn is_ready(batch: &Batch) -> bool {
        if batch.record_count() >= MAX_BATCH_SIZE {
            tracing::info!(
                "Batch '{:?}' size {} exceeds maximum {}",
                batch.partition(),
                batch.record_count(),
                MAX_BATCH_SIZE
            );
            return true;
        };

        let batch_age = Utc::now() - batch.oldest_record();
        if batch_age >= *MAX_BATCH_AGE {
            tracing::info!(
                "Batch '{:?}' age {} exceeds maximum {}",
                batch.partition(),
                batch_age.num_minutes(),
                MAX_BATCH_AGE.num_minutes()
            );
            return true;
        };
        false
    }
}
