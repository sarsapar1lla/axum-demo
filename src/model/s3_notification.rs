use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct S3Notification {
    #[serde(rename(deserialize = "Records"))]
    records: Vec<Record>,
}

impl S3Notification {
    pub fn records(&self) -> &[Record] {
        &self.records
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Record {
    #[serde(rename(deserialize = "eventTime"))]
    event_time: DateTime<Utc>,
    s3: S3,
}

impl Record {
    pub fn event_time(&self) -> &DateTime<Utc> {
        &self.event_time
    }

    pub fn s3(&self) -> &S3 {
        &self.s3
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct S3 {
    bucket: Bucket,
    object: Object,
}

impl S3 {
    pub fn bucket(&self) -> &Bucket {
        &self.bucket
    }

    pub fn object(&self) -> &Object {
        &self.object
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Bucket {
    name: String,
}

impl Bucket {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Object {
    key: String,
}

impl Object {
    pub fn key(&self) -> &str {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialises_s3_notification() {
        let actual: S3Notification = serde_json::from_str(notification()).unwrap();
        assert_eq!(actual, expected())
    }

    fn expected() -> S3Notification {
        let record = Record {
            event_time: DateTime::parse_from_rfc3339("2024-08-10T12:53:00.000+00:00")
                .unwrap()
                .to_utc(),
            s3: S3 {
                bucket: Bucket {
                    name: String::from("test-bucket"),
                },
                object: Object {
                    key: String::from("1234.json"),
                },
            },
        };
        S3Notification {
            records: vec![record],
        }
    }

    fn notification() -> &'static str {
        r#"{"Records":[{"eventTime":"2024-08-10T12:53:00.000Z","eventName":"S3:ObjectCreated","s3":{"bucket":{"name":"test-bucket"},"object":{"key":"1234.json"}}}]}"#
    }
}
