#[macro_use] extern crate rocket;
use aws_sdk_dynamodb::Client as DynamodbClient;


#[get("/")]
fn index() -> &'static str {
    let config = aws_config::from_env().region("us-east-1").load().await;
    let dynamodb_client = DynamodbClient::new(&config);
    let results = client
        .query()
        .table_name("cdemonchy-blog-stats")
        .send()
        .await?;
    if let Some(items) = results.items {
        let movies = items.iter().map(|v| v.into()).collect();
        return movies.join("-");
    } else {
        return "Nothing";
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
