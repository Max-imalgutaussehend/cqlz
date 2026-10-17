//! Front-end throughput on the same CQL: cqlc-core lexer/parser/full analysis vs the general-purpose
//! `sqlparser` crate.
//!
//! Caveat, stated on the website too: sqlparser parses SQL, not CQL. It is included only as a
//! widely used Rust parser baseline, restricted to the statements it accepts, and does **no** schema
//! resolution or type checking, whereas `cqlc_core::compile` resolves the schema and checks every query.

use cqlc_core::span::FileId;
use cqlc_core::{Input, compile, lexer, parser};
use cqlz_benchmarks::{QUERIES, SCHEMA, statements};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    let stmts = statements();
    let accepted: Vec<&String> = stmts
        .iter()
        .filter(|s| Parser::parse_sql(&GenericDialect {}, s).is_ok())
        .collect();
    eprintln!(
        "sqlparser accepts {}/{} statements; benchmarking those",
        accepted.len(),
        stmts.len()
    );
    let accepted_text: Vec<&str> = accepted.iter().map(|s| s.as_str()).collect();

    let mut g = c.benchmark_group("frontend");
    g.throughput(Throughput::Bytes(QUERIES.len() as u64));

    g.bench_function("cqlc_lex", |b| {
        b.iter(|| black_box(lexer::lex(FileId(0), black_box(QUERIES))))
    });
    g.bench_function("cqlc_parse_queries", |b| {
        b.iter(|| black_box(parser::parse_queries(FileId(0), black_box(QUERIES))))
    });
    g.bench_function("cqlc_compile_schema_and_queries", |b| {
        let schema = [Input::new("schema.cql", SCHEMA)];
        let queries = [Input::new("queries.cql", QUERIES)];
        b.iter(|| black_box(compile(&schema, &queries)))
    });

    let bytes: usize = accepted_text.iter().map(|s| s.len()).sum();
    g.throughput(Throughput::Bytes(bytes as u64));
    g.bench_function("sqlparser_parse_accepted_statements", |b| {
        b.iter(|| {
            for s in &accepted_text {
                black_box(Parser::parse_sql(&GenericDialect {}, black_box(s)).unwrap());
            }
        })
    });
    // Same statements through cqlc's parser, for an apples-to-apples parse comparison.
    let joined: String = accepted_text
        .iter()
        .map(|s| format!("-- name: Q :exec\n{s};\n"))
        .collect();
    g.bench_function("cqlc_parse_same_statements", |b| {
        b.iter(|| black_box(parser::parse_queries(FileId(0), black_box(&joined))))
    });
    g.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
