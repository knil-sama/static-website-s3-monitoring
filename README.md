# static-website-s3-monitoring
Analyse s3 access log and generate monitoring

# Cargo lambda

```bash
cargo lambda build
```

```bash
cargo lambda watch
```

```bash
cargo lambda invoke --data-file example_eventbridge_event.json 
```