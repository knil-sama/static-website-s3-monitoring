use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
use aws_sdk_s3::Client;
use http::StatusCode;
use s3_access_log_rust::convert_wsc_str_to_s3_access_log_record;
use std::collections::HashMap;
use tracing::{debug, info};
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
    bucket: &str,
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
                for object in output.contents() {
                    let key = object.key().unwrap();
                    let logs_to_parse = get_s3_file_content(&client, bucket, key).await;
                    let processed_logs =
                        convert_wsc_str_to_s3_access_log_record(&logs_to_parse.unwrap());
                    for log in processed_logs {
                        match log.operation.as_str() {
                            "WEBSITE.GET.OBJECT" => match log.http_status {
                                StatusCode::OK => {
                                    if log.key.ends_with(".html") {
                                        let counter =
                                            counter_page_access.entry(log.key).or_insert(0);
                                        *counter += 1;
                                    }
                                }
                                _ => {
                                    debug!("invalide status code");
                                }
                            },
                            other => {
                                debug!("not useful operation {other:?}");
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("{err:?}");
            }
        }
    }
    info!("{counter_page_access:?}");
    Ok(())
}