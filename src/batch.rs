use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use chrono::NaiveDate;

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
    json: String,
}

impl Entry {
    pub fn new(partition: Partition, json: &str) -> Self {
        Self {
            partition,
            json: String::from(json),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Batch {
    partition: Partition,
    records: Vec<String>,
}

impl Batch {
    pub fn partition(&self) -> &Partition {
        &self.partition
    }

    pub fn records(&self) -> &[String] {
        &self.records
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
            batches_lock
                .entry(entry.partition)
                .or_insert_with_key(|key| Batch {
                    partition: key.clone(),
                    records: Vec::new(),
                })
                .records
                .push(entry.json);
        }

        batches_lock.values().cloned().collect()
    }

    fn delete_batch(&self, partition: &Partition) {
        tracing::info!("Deleting batch '{:?}'", partition);
        let mut batches_lock = self.batches.lock().unwrap();
        batches_lock.remove(partition);
    }
}
