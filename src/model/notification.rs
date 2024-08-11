use chrono::DateTime;
use chrono::Utc;

#[derive(Debug)]
pub struct Notification {
    message_id: String,
    receipt_handle: String,
    created: DateTime<Utc>,
    bucket: String,
    key: String,
}

impl Notification {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn message_id(&self) -> &str {
        &self.message_id
    }

    pub fn receipt_handle(&self) -> &str {
        &self.receipt_handle
    }

    pub fn created(&self) -> &DateTime<Utc> {
        &self.created
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

pub struct Builder {
    message_id: Option<String>,
    receipt_handle: Option<String>,
    created: Option<DateTime<Utc>>,
    bucket: Option<String>,
    key: Option<String>,
}

impl Builder {
    fn new() -> Self {
        Builder {
            message_id: None,
            receipt_handle: None,
            created: None,
            bucket: None,
            key: None,
        }
    }

    pub fn message_id(mut self, message_id: &str) -> Self {
        self.message_id.replace(String::from(message_id));
        self
    }

    pub fn receipt_handle(mut self, receipt_handle: &str) -> Self {
        self.receipt_handle.replace(String::from(receipt_handle));
        self
    }

    pub fn created(mut self, created: DateTime<Utc>) -> Self {
        self.created.replace(created);
        self
    }

    pub fn bucket(mut self, bucket: &str) -> Self {
        self.bucket.replace(String::from(bucket));
        self
    }

    pub fn key(mut self, key: &str) -> Self {
        self.key.replace(String::from(key));
        self
    }

    pub fn build(self) -> Notification {
        Notification {
            message_id: self.message_id.unwrap(),
            receipt_handle: self.receipt_handle.unwrap(),
            created: self.created.unwrap(),
            bucket: self.bucket.unwrap(),
            key: self.key.unwrap(),
        }
    }
}
