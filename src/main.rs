use std::{env, sync::Arc};

use axum::{routing::get, Router};
use deleter::SqsMessageDeleter;
use handler::EventHandler;
use processor::NotificationProcessorImpl;
use schedule::{schedule_batch_writer, schedule_handler};
use supplier::SqsSupplier;
use tokio_util::sync::CancellationToken;
use writer::{BatchWriter, S3Writer};

mod batch;
mod deleter;
mod handler;
mod model;
mod processor;
mod schedule;
mod supplier;
mod writer;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::default();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let localstack_endpoint = env::var("LOCALSTACK_ENDPOINT").expect("Endpoint should be provided");
    let queue_url = env::var("INPUT_QUEUE_URL").expect("Input queue url should be provided");
    let output_bucket =
        env::var("OUTPUT_BUCKET_NAME").expect("Output bucket name should be provided");

    let sqs_config = aws_sdk_sqs::Config::builder()
        .endpoint_url(&localstack_endpoint)
        .region(aws_sdk_sqs::config::Region::new("us-east-1"))
        .credentials_provider(aws_sdk_sqs::config::Credentials::new(
            "secret", "secret", None, None, "static",
        ))
        .behavior_version_latest()
        .build();
    let sqs_client = aws_sdk_sqs::Client::from_conf(sqs_config);

    let s3_config = aws_sdk_s3::Config::builder()
        .endpoint_url(&localstack_endpoint)
        .region(aws_sdk_s3::config::Region::new("us-east-1"))
        .force_path_style(true)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            "secret", "secret", None, None, "static",
        ))
        .behavior_version_latest()
        .build();
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

    let supplier = SqsSupplier::new(sqs_client.clone(), &queue_url);
    let batch_store = Arc::new(batch::StoreImpl::new());
    let processor =
        NotificationProcessorImpl::new(Box::new(s3_client.clone()), batch_store.clone());
    let deleter = SqsMessageDeleter::new(sqs_client, &queue_url);

    let handler = EventHandler::new(Arc::new(supplier), Arc::new(processor), Arc::new(deleter));

    let writer = S3Writer::new(s3_client, &output_bucket);
    let batch_writer = Arc::new(BatchWriter::new(batch_store, Box::new(writer)));
    let shutdown_token = CancellationToken::new();

    schedule_handler(handler, shutdown_token.child_token());
    schedule_batch_writer(batch_writer, shutdown_token.child_token());

    let app = Router::new().route("/ping", get(ping));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn ping() -> &'static str {
    "pong"
}
