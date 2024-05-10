use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
use http::StatusCode;
use aws_sdk_dynamodb::Client as DynamodbClient;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_s3::Client as S3Client;
use s3_access_log_rust::convert_wsc_str_to_s3_access_log_record;
use futures::future;
use lambda_runtime::{service_fn, run, LambdaEvent};
use aws_lambda_events::encodings::Error;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use aws_lambda_events::cloudwatch_events::CloudWatchEvent;
use chrono::{DateTime, Utc};

async fn get_s3_file_content(
    client: &S3Client,
    bucket: &str,
    key: &str,
) -> Result<String, anyhow::Error> {
    let object = client.get_object().bucket(bucket).key(key).send().await?;

    let bytes = object.body.collect().await?.into_bytes();
    let text = std::str::from_utf8(&bytes)?;

    Ok(text.to_owned())
}

fn get_iterator_s3_objects(
    client: &S3Client,
    bucket: &str
) -> aws_smithy_async::future::pagination_stream::PaginationStream<
    Result<
        ListObjectsV2Output,
        aws_smithy_runtime_api::client::result::SdkError<
            ListObjectsV2Error,
            aws_smithy_runtime_api::http::Response,
        >,
    >,
> {
    client
        .list_objects_v2()
        .bucket(bucket.to_owned())
        .into_paginator()
        .send()
}

fn is_log_from_s3_static_page(operation: &str, http_status: StatusCode, filename: &str) -> bool {
    return operation.eq("WEBSITE.GET.OBJECT") && http_status == StatusCode::OK && filename.ends_with(".html")
}

fn mapping_page_name(full_path: String) -> String {
    match full_path.as_str() {
        "index.html" => "cv".to_string(),
        _ =>  {
            let mut splitted = full_path.split('/');
            let penultimate = splitted.clone().count()-2;
            splitted.nth(penultimate).unwrap().to_owned()
        }
    }
}

async fn update_database(client: &DynamodbClient, table_name: &str, page_name: String, time: DateTime<Utc>) -> Result<String, anyhow::Error>{
    let page_name_dynamodb = AttributeValue::S(page_name);
    let time_dynamodb = AttributeValue::S(time.to_rfc3339());

    let request = client
        .put_item()
        .table_name(table_name)
        .item("page_name", page_name_dynamodb)
        .item("time", time_dynamodb);
    request.send().await?;
    Ok("done".to_owned())

}
/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
#[allow(clippy::result_large_err)]
async fn function_handler(_event: LambdaEvent<CloudWatchEvent>) -> Result<(), Error> {
    // Extract some useful information from the request
    tracing::info!("Rust function invoked");
    let config = aws_config::from_env().region("us-east-1").load().await;
    let s3_client = S3Client::new(&config);
    let dynamodb_client = DynamodbClient::new(&config);
    let bucket = "cdemonchy-logs-us-east-1";
    let table = "cdemonchy-blog-stats";
    let mut iterator_s3_object = get_iterator_s3_objects(&s3_client, bucket);
    tracing::info!("Starting iteration");
    while let Some(result) = iterator_s3_object.next().await {
        match result {
            Ok(output) => {
                let futures = output.contents().iter().map(|object| get_s3_file_content(&s3_client, bucket,object.key().unwrap())).collect::<Vec<_>>();
                let logs_to_parse: Vec<_> = future::join_all(futures).await;
                // use filter map because we got case were there is no time to in s3 access log and we can´t do anything with it
                let processed_logs: Vec<_> = logs_to_parse.iter().filter_map(|object| Some(convert_wsc_str_to_s3_access_log_record(object.as_ref().unwrap(), true))).collect::<Vec<_>>().into_iter().flatten().collect();
                let valid_logs: Vec<_> = processed_logs.iter().filter(|log| is_log_from_s3_static_page(&log.operation, log.http_status, &log.key)).collect();
                let other_futures = valid_logs.iter().map(|log| update_database(&dynamodb_client, &table, mapping_page_name(log.key.clone()), log.time.clone())).collect::<Vec<_>>();
                let _: Vec<_> = future::join_all(other_futures).await;

            }
            Err(err) => {
                eprintln!("{err:?}");
            }
        }
    }
    tracing::info!("Ending iteration");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
