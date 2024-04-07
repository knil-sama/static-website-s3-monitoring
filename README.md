# static-website-s3-monitoring
Analyse s3 access log and generate monitoring

# Infra requirements

On same AWS account
* AWS S3 static website
* On said s3 bucket activate s3 access logs
* A lambda for reading log and writing it to dynamodb (this code)
* Add dynamodb as target for metrics aws dynamodb, example with the following command 
```bash
aws --region us-east-1 dynamodb create-table \
    --table-name cdemonchy-blog-stats \
    --attribute-definitions \
        AttributeName=page_name,AttributeType=S \
        AttributeName=time,AttributeType=S \
    --key-schema \
        AttributeName=page_name,KeyType=HASH \
        AttributeName=time,KeyType=RANGE \
    --provisioned-throughput \
        ReadCapacityUnits=1,WriteCapacityUnits=1 \
    --table-class STANDARD
```
# Cargo lambda

```bash
cargo lambda build
cargo lambda watch
```

```bash
cargo lambda invoke --data-file example_eventbridge_event.json 
```