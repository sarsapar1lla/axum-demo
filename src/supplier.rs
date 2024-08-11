use aws_sdk_sqs::{types::Message, Client};
use axum::async_trait;

use crate::model::{Record, S3Notification};

use super::model::Notification;

#[async_trait]
pub trait Supplier {
    async fn get(&self) -> Vec<Notification>;
}

pub struct SqsSupplier {
    client: Client,
    queue_url: String,
}

impl SqsSupplier {
    pub fn new(client: Client, queue_url: &str) -> Self {
        Self {
            client,
            queue_url: String::from(queue_url),
        }
    }
}

#[async_trait]
impl Supplier for SqsSupplier {
    async fn get(&self) -> Vec<Notification> {
        let response = self
            .client
            .receive_message()
            .queue_url(&self.queue_url)
            .max_number_of_messages(10)
            .wait_time_seconds(5)
            .send()
            .await
            .unwrap();

        tracing::info!(
            "Received {} messages from '{}'",
            response.messages().len(),
            self.queue_url
        );
        response
            .messages()
            .iter()
            .flat_map(notifications_from)
            .collect()
    }
}

fn notifications_from(message: &Message) -> Vec<Notification> {
    let s3_notification: S3Notification = serde_json::from_str(message.body().unwrap()).unwrap();
    s3_notification
        .records()
        .iter()
        .map(|record| notification_from(record, message))
        .collect()
}

fn notification_from(record: &Record, message: &Message) -> Notification {
    Notification::builder()
        .message_id(message.message_id().unwrap())
        .receipt_handle(message.receipt_handle().unwrap())
        .created(*record.event_time())
        .bucket(record.s3().bucket().name())
        .key(record.s3().object().key())
        .build()
}
