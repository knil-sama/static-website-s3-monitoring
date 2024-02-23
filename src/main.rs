use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
use aws_sdk_s3::Client;
use http::StatusCode;
use s3_access_log_rust::convert_wsc_str_to_s3_access_log_record;
use std::collections::HashMap;
use tracing::{info};
use futures::future;
use plotters::prelude::*;
use lambda_runtime::{service_fn, run, LambdaEvent};
use aws_lambda_events::encodings::Error;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use aws_lambda_events::cloudwatch_events::CloudWatchEvent;

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

fn generate_graph(data: HashMap::<String, u32>) -> Result<(), Box<dyn std::error::Error>> {
    const OUT_FILE_NAME: &str = "histogram.png";
    let root = BitMapBackend::new(OUT_FILE_NAME, (640, 480)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Cdemonchy.com stats", ("sans-serif", 50.0))
        .build_cartesian_2d(0..(data.keys().len()-1) as u32, 0u32..*data.values().max().unwrap())?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("page")
        .x_label_formatter(&|x| format!("{:?}",data.clone().into_keys().collect::<Vec<String>>()[*x as usize]))
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(data.values().enumerate().map(|(x, y)| (x as u32, *y)))
            //.data(data.vqlues().map(|x: &u32| (*x, 1))),
    )?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}

fn mapping_page_name(full_path: String) -> String {
    match full_path.as_str() {
        "index.html" => "csv".to_string(),
        _ =>  {
            let mut splitted = full_path.split('/');
            let penultimate = splitted.clone().count()-2;
            splitted.nth(penultimate).unwrap().to_owned()
        }
    }
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
#[allow(clippy::result_large_err)]
async fn function_handler(_event: LambdaEvent<CloudWatchEvent>) -> Result<(), Error> {
    // Extract some useful information from the request

    let config = aws_config::from_env().region("us-east-1").load().await;
    let client = Client::new(&config);
    let bucket = "cdemonchy-logs-us-east-1";
    let mut iterator_s3_object = get_iterator_s3_objects(&client, bucket);
    let mut counter_page_access = HashMap::<String, u32>::new();
    while let Some(result) = iterator_s3_object.next().await {
        match result {
            Ok(output) => {
                let futures = output.contents().iter().map(|object| get_s3_file_content(&client, bucket,object.key().unwrap())).collect::<Vec<_>>();
                let logs_to_parse: Vec<_> = future::join_all(futures).await;
                let processed_logs: Vec<_> = logs_to_parse.iter().map(|object| convert_wsc_str_to_s3_access_log_record(object.as_ref().unwrap())).collect::<Vec<_>>().into_iter().flatten().collect();
                let valid_logs: Vec<_> = processed_logs.iter().filter(|log| log.operation.eq("WEBSITE.GET.OBJECT") && log.http_status == StatusCode::OK && log.key.ends_with(".html")).collect();
                valid_logs.iter().for_each(|log| *(counter_page_access.entry(mapping_page_name(log.key.clone())).or_insert(0))+= 1);
            }
            Err(err) => {
                eprintln!("{err:?}");
            }
        }
    }
    info!("{counter_page_access:?}");
    let _ = generate_graph(counter_page_access);
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
