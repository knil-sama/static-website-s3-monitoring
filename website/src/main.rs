use aws_sdk_dynamodb::Client as DynamodbClient;
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::types::{AttributeValue};
//use aws_smithy_types::error::operation::BuildError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use lambda_http::{run, http::{StatusCode, Response},Body, service_fn, Error, IntoResponse, Request, RequestPayloadExt};
use std::collections::HashMap;
use thiserror::Error;
use counter::Counter;
//use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use serde_json::json;
use rocket::{self, get, routes};
//use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};

#[derive(Error, Debug)]
pub enum PageAccessError {
    #[error("failed to parse serde_json::Value into PageAccess {0}")]
    FromValue(&'static Value),

    #[error("failed to parse response into PageAccess: {0}")]
    FromSerde(serde_dynamo::Error),

    #[error("aws_sdk_dynamodb error: {0}")]
    Dynamo(aws_sdk_dynamodb::Error),

    #[error("unknown DynamoDB PageAccess error: {0}")]
    Unknown(String),
}

impl From<aws_sdk_dynamodb::Error> for PageAccessError {
    fn from(err: aws_sdk_dynamodb::Error) -> Self {
        PageAccessError::Dynamo(err)
    }
}

impl From<serde_dynamo::Error> for PageAccessError {
    fn from(err: serde_dynamo::Error) -> Self {
        PageAccessError::FromSerde(err)
    }
}

impl<E, R> From<SdkError<E, R>> for PageAccessError
where
    E: std::fmt::Debug,
    R: std::fmt::Debug,
{
    fn from(err: SdkError<E, R>) -> Self {
        PageAccessError::Unknown(format!("{err:?}"))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageAccess {
    page_name: String,
    time: String,
}

impl PageAccess {
    pub fn new(page_name: String, time: String) -> Self {
        PageAccess {
            page_name,
            time
        }
    }
}

fn as_string(val: Option<&AttributeValue>, default: &String) -> String {
    if let Some(v) = val {
        if let Ok(s) = v.as_s() {
            return s.to_owned();
        }
    }
    default.to_owned()
}

impl From<&HashMap<String, AttributeValue>> for PageAccess {
    fn from(value: &HashMap<String, AttributeValue>) -> Self {
        let page_access = PageAccess::new(
            as_string(value.get("page_name"), &"".to_string()),
            as_string(value.get("time"), &"".to_string()),
        );
        page_access
    }
}
pub async fn page_access() -> Result<Vec<PageAccess>, PageAccessError> {
    let config = aws_config::from_env().region("us-east-1").load().await;
    let dynamodb_client = DynamodbClient::new(&config);
    let results = dynamodb_client
        .scan()
        .table_name("cdemonchy-blog-stats")
        .send()
        .await?;
    if let Some(items) = results.items {
        let page_scan = items.iter().map(|v| v.into()).collect();
        Ok(page_scan)
    } else {
        Ok(vec![])
    }
}


async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body("Hello AWS Lambda HTTP request".into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
    run(service_fn(function_handler)).await
}

#[get("/stats")]
async fn stats() -> String {
    let res: Vec<PageAccess>  = match page_access().await {
        Ok(page_access) => page_access,
        Err(_error) => return "Can´t reach database".to_string(), // should swap error and generic message based on env
    };
    println!("{res:?}");
    let counter_page = res.iter().map(|x| &x.page_name).collect::<Counter<_>>();
    println!("{counter_page:?}");
    json!(*counter_page).to_string()
}
