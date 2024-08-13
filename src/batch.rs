use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use chrono::{DateTime, NaiveDate, Utc};

pub use summary::{Summariser, Summary};

mod summary;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Partition {
    source: String,
    date: NaiveDate,
}

impl Partition {
    pub fn new(source: &str, date: NaiveDate) -> Self {
        Self {
            source: String::from(source),
            date,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    partition: Partition,
    created: DateTime<Utc>,
    json: String,
}

impl Entry {
    pub fn new(partition: Partition, created: &DateTime<Utc>, json: &str) -> Self {
        Self {
            partition,
            created: *created,
            json: String::from(json),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Batch {
    partition: Partition,
    oldest_record: DateTime<Utc>,
    records: Vec<String>,
}

impl Batch {
    pub fn partition(&self) -> &Partition {
        &self.partition
    }

    pub fn oldest_record(&self) -> &DateTime<Utc> {
        &self.oldest_record
    }

    pub fn records(&self) -> &[String] {
        &self.records
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

pub trait Store {
    fn add(&self, entry: Entry);

    fn batches(&self) -> Vec<Batch>;

    fn delete_batch(&self, partition: &Partition);
}

pub struct StoreImpl {
    queue: Mutex<VecDeque<Entry>>,
    batches: Mutex<HashMap<Partition, Batch>>,
}

impl StoreImpl {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            batches: Mutex::new(HashMap::new()),
        }
    }
}

impl Store for StoreImpl {
    fn add(&self, entry: Entry) {
        self.queue.lock().unwrap().push_back(entry);
    }

    fn batches(&self) -> Vec<Batch> {
        let mut queue_lock = self.queue.lock().unwrap();
        let entries = queue_lock.drain(..);

        let mut batches_lock = self.batches.lock().unwrap();

        for entry in entries {
            let batch = batches_lock
                .entry(entry.partition)
                .or_insert_with_key(|key| Batch {
                    partition: key.clone(),
                    oldest_record: Utc::now(),
                    records: Vec::new(),
                });

            batch.records.push(entry.json);
            if entry.created < batch.oldest_record {
                batch.oldest_record = entry.created;
            };
        }

        batches_lock.values().cloned().collect()
    }

    fn delete_batch(&self, partition: &Partition) {
        tracing::info!("Deleting batch '{:?}'", partition);
        let mut batches_lock = self.batches.lock().unwrap();
        batches_lock.remove(partition);
    }
}
