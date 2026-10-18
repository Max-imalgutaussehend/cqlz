//! cql-ai cost per statement vs running cqlc's analyzer on the same statements.
//!
//! The model is randomly initialised: latency does not depend on the weights.

use burn::prelude::*;
use cql_ai_model::features::{DataStats, Facts, context, facts};
use cql_ai_model::model::{ModelConfig, batch_ctx, batch_tokens};
use cql_ai_model::tokenizer::{SchemaInfo, encode};
use cqlc_core::{Input, compile};
use cqlz_benchmarks::{SCHEMA, statements};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    let device = Device::flex();
    let model = ModelConfig::new().init(&device);
    let schema_info = SchemaInfo::from_ddl(SCHEMA);
    let stats = DataStats {
        partitions: Some(1e5),
        rows_per_partition: Some(100.0),
        ..Default::default()
    };
    let stmts = statements();
    let n = stmts.len();

    let ctx: Vec<_> = stmts
        .iter()
        .map(|s| context(Some(&stats), &Facts { ..facts(s) }))
        .collect();
    let ids: Vec<Vec<u32>> = stmts.iter().map(|s| encode(s, &schema_info)).collect();
    let seqs: Vec<&[u32]> = ids.iter().map(Vec::as_slice).collect();

    let mut g = c.benchmark_group("per_file_of_statements");
    g.bench_function(BenchmarkId::new("cql_ai_tokenize", n), |b| {
        b.iter(|| {
            for s in &stmts {
                black_box(encode(black_box(s), &schema_info));
                black_box(facts(s));
            }
        })
    });
    g.bench_function(BenchmarkId::new("cql_ai_forward_one_batch", n), |b| {
        b.iter(|| {
            let (p, cost) = model.predict(
                batch_tokens(black_box(&seqs), &device),
                batch_ctx(&ctx, &device),
            );
            black_box((p.into_data(), cost.into_data()))
        })
    });
    let schema = [Input::new("schema.cql", SCHEMA)];
    let queries = [Input::new("queries.cql", cqlz_benchmarks::QUERIES)];
    g.bench_function(
        BenchmarkId::new("cqlc_compile_schema_and_queries", n),
        |b| b.iter(|| black_box(compile(&schema, &queries))),
    );
    g.finish();

    c.bench_function("cql_ai_forward_single_statement", |b| {
        b.iter(|| {
            let (p, cost) = model.predict(
                batch_tokens(&[ids[0].as_slice()], &device),
                batch_ctx(&ctx[..1], &device),
            );
            black_box((p.into_data(), cost.into_data()))
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
