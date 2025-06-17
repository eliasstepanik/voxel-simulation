# Run tests in release mode

```bash
cargo test --release
```

* Compiles with optimizations (`--release`).
* Executes all test binaries from `target/release/deps`.
* Use for performance‑sensitive benchmarks.
* Build takes longer; cache artifacts in CI.
