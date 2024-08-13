use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Summary {
    source: String,
    date: NaiveDate,
    oldest_record: DateTime<Utc>,
    record_count: usize,
}

pub struct Summariser {
    batch_store: Arc<dyn super::Store + Sync + Send>,
}

impl Summariser {
    pub fn new(batch_store: Arc<dyn super::Store + Sync + Send>) -> Self {
        Self { batch_store }
    }

    pub fn summary(&self) -> Vec<Summary> {
        self.batch_store
            .batches()
            .into_iter()
            .map(From::from)
            .collect()
    }
}

impl From<super::Batch> for Summary {
    fn from(value: super::Batch) -> Self {
        Self {
            source: value.partition().source().to_owned(),
            date: value.partition().date().to_owned(),
            oldest_record: value.oldest_record().to_owned(),
            record_count: value.record_count(),
        }
    }
}
