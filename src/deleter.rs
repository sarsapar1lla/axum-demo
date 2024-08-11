use aws_sdk_sqs::Client;
use axum::async_trait;

#[async_trait]
pub trait MessageDeleter {
    async fn delete(&self, receipt_handle: &str);
}

pub struct SqsMessageDeleter {
    client: Client,
    queue_url: String,
}

impl SqsMessageDeleter {
    pub fn new(client: Client, queue_url: &str) -> Self {
        Self {
            client,
            queue_url: String::from(queue_url),
        }
    }
}

#[async_trait]
impl MessageDeleter for SqsMessageDeleter {
    async fn delete(&self, receipt_handle: &str) {
        self.client
            .delete_message()
            .queue_url(&self.queue_url)
            .receipt_handle(receipt_handle)
            .send()
            .await
            .unwrap();
    }
}
