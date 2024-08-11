use aws_sdk_s3::Client;
use axum::async_trait;

use crate::model::Notification;

#[async_trait]
pub trait EventExtractor {
    async fn extract(&self, notification: &Notification) -> Vec<u8>;
}

#[async_trait]
impl EventExtractor for Client {
    async fn extract(&self, notification: &Notification) -> Vec<u8> {
        self.get_object()
            .bucket(notification.bucket())
            .key(notification.key())
            .send()
            .await
            .unwrap()
            .body
            .collect()
            .await
            .unwrap()
            .to_vec()
    }
}
