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


     

sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu

RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --package website --target aarch64-unknown-linux-gnu


aws lambda delete-function --region us-east-1 \
  --function-name website-bis-bis

cp target/aarch64-unknown-linux-gnu/release/bootstrap ./bootstrap
chmod +x ./bootstrap
zip function.zip bootstrap

aws lambda create-function \
     --region us-east-1 \
     --function-name website-bis-bis \
     --runtime provided.al2023 \
     --role arn:aws:iam::650593633156:role/cargo-lambda-role-dfae10c4-51cb-4cf3-8af6-a66dba6ccb7d \
     --handler rust.handler \
     --zip-file fileb://function.zip

aws lambda delete-function-url-config \
     --region us-east-1 \
     --function-name website-bis-bis

aws lambda create-function-url-config \
     --region us-east-1 \
     --auth-type NONE \
     --function-name website-bis-bis


     
