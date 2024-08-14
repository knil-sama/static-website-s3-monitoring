# Code code requirements

Install rust [tooling](https://www.rust-lang.org/tools/install)
Install cargo lambda executable

```bash
cargo install cargo-lambda
```

# Cargo deployment

In a terminal run

```bash
cargo lambda build --release
cargo lambda deploy
```

# Cargo lambda testing

You need to have credentials for aws already loaded in your machine

In a terminal run

```bash
cargo lambda build
cargo lambda watch
```

<!> You can get an "ERROR command killed code=ForceStop" showing up and just ignore it

In another run this to send data

```bash
cargo lambda invoke --data-file example_eventbridge_event.json
```