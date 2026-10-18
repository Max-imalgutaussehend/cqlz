# cqlz

The **cqlz family**: typesafe tools for Cassandra and ScyllaDB in Rust.

| Project | Direction | |
|---|---|---|
| [`cqlc`](https://github.com/Max-imalgutaussehend/cqlc) | schema-first | CQL files → typed Rust (like sqlc) |
| [`cqlx`](https://github.com/Max-imalgutaussehend/cqlx) | code-first | Rust structs → schema + type-state query builders |
| [`cql-ai`](https://github.com/Max-imalgutaussehend/cql-ai) | performance | CQL HPC: predicts performance bottlenecks in CI, trained on measured workloads |
| [`cqlz-gym`](https://github.com/Max-imalgutaussehend/cqlz-lab) | design | Designs Cassandra tables for a workload (ILP + learned policy, verified by cqlc-core) |
| [`cqlint`](https://github.com/Max-imalgutaussehend/cqlint) | lint + format | A ruff for CQL: formatter and 13 performance and safety rules |
| [`cql-migrate`](https://github.com/Max-imalgutaussehend/cql-migrate) | migrations | Checksummed, locked schema migrations with offline chain check, diff and drift detection |
| [`cqlprof`](https://github.com/Max-imalgutaussehend/cqlprof) | profiling | EXPLAIN for CQL: tracing-based profiler for Cassandra and ScyllaDB |

This repository holds the family website (`site/`, deployed with GitHub Pages) and the cross-project
benchmarks (`benchmarks/`). Results are on the *Benchmarks* page of the site.

```sh
cargo bench -p cqlz-benchmarks --bench frontend    # cqlc-core vs sqlparser
cargo bench -p cqlz-benchmarks --bench inference   # cql-ai latency
cargo run --release --bin token_stats              # sequence-length comparison
```

Site sources are in `site/src/*.html`; `python3 site/partials.py` wraps them with the shared header and footer.

Apache-2.0
