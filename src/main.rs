use std::{env, sync::Arc, time::Duration};

use axum::{routing::get, Json, Router};
use deleter::SqsMessageDeleter;
use handler::EventHandler;
use processor::NotificationProcessorImpl;
use supplier::SqsSupplier;
use tokio::{
    net::TcpListener,
    time::{interval_at, Instant},
};
use writer::{BatchWriter, S3Writer};

mod batch;
mod deleter;
mod handler;
mod model;
mod processor;
mod schedule;
mod shutdown;
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
    let batch_writer = Arc::new(BatchWriter::new(batch_store.clone(), Box::new(writer)));

    let (shutdown_send, _) = tokio::sync::broadcast::channel::<()>(1);
    let handler_task = schedule::task(
        Arc::new(handler),
        interval_at(Instant::now(), Duration::from_millis(5_000)),
        shutdown_send.subscribe(),
    );
    let writer_task = schedule::task(
        batch_writer.clone(),
        interval_at(
            Instant::now() + Duration::from_millis(2_000),
            Duration::from_millis(20_000),
        ),
        shutdown_send.subscribe(),
    );
    let background_tasks = vec![handler_task, writer_task];

    let summariser = Arc::new(batch::Summariser::new(batch_store));

    let app = Router::new()
        .route("/ping", get(ping))
        .route("/batch/summary", get(move || summary(summariser)));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown::hook(
            shutdown_send,
            batch_writer,
            background_tasks,
        ))
        .await
        .unwrap();
}

async fn ping() -> &'static str {
    "pong"
}

async fn summary(summariser: Arc<batch::Summariser>) -> Json<Vec<batch::Summary>> {
    let summaries = summariser.summary();
    Json(summaries)
}
