use aws_sdk_s3::{primitives::ByteStream, Client};
use axum::async_trait;

mod batch;

use crate::batch::{Batch, Partition};
pub use batch::BatchWriter;

#[async_trait]
pub trait Writer {
    async fn write(&self, batch: &Batch);
}

pub struct S3Writer {
    client: Client,
    bucket: String,
    key_function: fn() -> String,
}

impl S3Writer {
    pub fn new(client: Client, bucket: &str) -> Self {
        Self {
            client,
            bucket: String::from(bucket),
            key_function: generate_id,
        }
    }

    fn key(&self, partition: &Partition) -> String {
        format!(
            "output/source={}/date={}/{}.jsonl",
            partition.source(),
            partition.date(),
            (self.key_function)()
        )
    }
}

#[async_trait]
impl Writer for S3Writer {
    async fn write(&self, batch: &Batch) {
        let key = self.key(batch.partition());
        let content = file_content_from(batch.records());

        tracing::info!(
            "Writing batch '{:?}' to 's3://{}/{}'",
            batch.partition(),
            self.bucket,
            key
        );
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::new(content.into()))
            .send()
            .await
            .unwrap();
    }
}

fn file_content_from(records: &[String]) -> Vec<u8> {
    records.join("\n").into_bytes()
}

fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
