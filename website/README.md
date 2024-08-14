```bash
cargo lambda build
cargo lambda watch
```

In another terminal

```bash
cargo lambda invoke --data-file apigw_no_host.json
```