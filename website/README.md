```bash
cargo lambda build --release --arm64
cargo lambda watch
```

In another terminal

```bash
cargo lambda invoke --data-file lambda_url_http.json
```

```bash
cargo lambda deploy website
```

=> This don't create release properly
docker build --tag 'rust-lambda-website' .

docker run -it --rm \
  -v ~/.cargo/registry:/root/.cargo/registry:z \
  -v .:/build:z \
  rust-lambda-website

this reuse cargo build and have same bug than before regaring parsing of lambda event
  aws lambda create-function \
     --region us-east-1 \
     --function-name website-bis \
     --runtime provided.al2023 \
     --role arn:aws:iam::650593633156:role/cargo-lambda-role-dfae10c4-51cb-4cf3-8af6-a66dba6ccb7d \
     --handler rust.handler \
     --zip-file fileb://target/lambda/website/bootstrap.zip