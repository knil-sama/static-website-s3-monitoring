use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
use aws_sdk_s3::Client;
use http::StatusCode;
use s3_access_log_rust::convert_wsc_str_to_s3_access_log_record;
use std::collections::HashMap;
use tracing::{debug, info};
use futures::future;

async fn get_s3_file_content(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<String, anyhow::Error> {
    let object = client.get_object().bucket(bucket).key(key).send().await?;

    let bytes = object.body.collect().await?.into_bytes();
    let text = std::str::from_utf8(&bytes)?;

    Ok(text.to_owned())
}

fn get_iterator_s3_objects(
    client: &Client,
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

#[tokio::main]
async fn main() -> Result<(), aws_sdk_s3::Error> {
    tracing_subscriber::fmt::init();

    let config = aws_config::from_env().region("us-east-1").load().await;
    let client = Client::new(&config);
    let bucket = "cdemonchy-logs-us-east-1";
    let mut iterator_s3_object = get_iterator_s3_objects(&client, bucket);
    let mut counter_page_access = HashMap::<String, u64>::new();
    while let Some(result) = iterator_s3_object.next().await {
        info!("start iterator");
        match result {
            Ok(output) => {
                let futures = output.contents().iter().map(|object| get_s3_file_content(&client, bucket,object.key().unwrap())).collect::<Vec<_>>();
                let logs_to_parse: Vec<_> = future::join_all(futures).await;
                let processed_logs: Vec<_> = logs_to_parse.iter().map(|object| convert_wsc_str_to_s3_access_log_record(object.as_ref().unwrap())).collect::<Vec<_>>().into_iter().flatten().collect();
                let valid_logs: Vec<_> = processed_logs.iter().filter(|log| log.operation.eq("WEBSITE.GET.OBJECT") && log.http_status == StatusCode::OK && log.key.ends_with(".html")).collect();
                valid_logs.iter().for_each(|log| *(counter_page_access.entry(log.key.to_owned()).or_insert(0))+= 1);
            }
            Err(err) => {
                eprintln!("{err:?}");
            }
        }
    }
    info!("{counter_page_access:?}");
    Ok(())
}
