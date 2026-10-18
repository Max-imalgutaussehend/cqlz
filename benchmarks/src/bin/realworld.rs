//! Real-world check: the KillrVideo reference schemas and query examples (Apache-2.0) through
//! `cqlc-core`, and the trained `cql-ai` model against `cqlc`'s verdicts on the same statements.
//!
//!     cargo run --release --bin realworld -- <cql-ai model dir>

use burn::prelude::*;
use cql_ai_model::io::load;
use cql_ai_model::model::batch_tokens;
use cql_ai_model::tokenizer::{SchemaInfo, encode, statements};
use cql_ai_model::LABELS;
use cqlc_core::diagnostic::DiagnosticCode as C;
use cqlc_core::{Input, compile};
use std::collections::BTreeMap;

fn split(src: &str) -> Vec<String> {
    statements(src).into_iter().map(|s| src[s.range].to_string()).collect()
}

fn first_word(s: &str) -> String {
    s.split(|c: char| !c.is_ascii_alphabetic()).next().unwrap_or("").to_ascii_lowercase()
}

fn label_of(code: C) -> Option<usize> {
    match code {
        C::IncompleteKey | C::BroadScan => Some(0),
        C::AllowFiltering => Some(1),
        C::NonKeyRestriction => Some(2),
        C::ClusteringGap => Some(3),
        C::OrderBy => Some(4),
        _ => None,
    }
}

fn main() {
    let model_dir = std::env::args().nth(1);
    let dir = "data/real";
    println!("== schemas through cqlc-core ==");
    let mut accepted_ddl: Vec<String> = Vec::new();
    let strip_default = |t: &str| -> String {
        let mut out = String::new();
        let mut rest = t;
        while let Some(i) = rest.find(" DEFAULT ") {
            out.push_str(&rest[..i]);
            let tail = &rest[i + 9..];
            // skip `fn()` or a literal up to the next comma / newline
            let end = tail.find([',', '\n']).unwrap_or(tail.len());
            rest = &tail[end..];
        }
        out.push_str(rest);
        out
    };
    for v in ["v3", "v4", "v5"] {
        let src = std::fs::read_to_string(format!("{dir}/kv-schema-{v}.cql")).unwrap();
        let tables: Vec<String> = split(&src).into_iter().filter(|s| first_word(s) == "create" && s.to_ascii_lowercase().contains("create table")).collect();
        let mut ok = 0;
        let mut why: BTreeMap<String, usize> = BTreeMap::new();
        let mut good: Vec<String> = Vec::new();
        for t in &tables {
            let c = compile(&[Input::new("s.cql", format!("{};", strip_default(t)))], &[]);
            match c.diagnostics.iter().find(|d| d.is_error()) {
                None => {
                    ok += 1;
                    good.push(format!("{};", strip_default(t)));
                }
                Some(d) => *why.entry(d.message.chars().take(60).collect()).or_default() += 1,
            }
        }
        let normalised = tables.iter().filter(|t| compile(&[Input::new("s.cql", format!("{};", strip_default(t)))], &[]).diagnostics.iter().all(|d| !d.is_error())).count();
        println!("schema {v}: {ok}/{} CREATE TABLE statements accepted as published, {normalised}/{} after removing `DEFAULT` clauses", tables.len(), tables.len());
        for (m, n) in why.iter().take(3) {
            println!("    rejected x{n}: {m}");
        }
        if v == "v4" {
            accepted_ddl = good;
        }
    }

    println!("\n== query examples (v4) against the accepted v4 tables ==");
    let schema_text = accepted_ddl.join("\n").replace("killrvideo.", "");
    let qsrc = std::fs::read_to_string(format!("{dir}/kv-schema-v4-query-examples.cql")).unwrap().replace("killrvideo.", "");
    // `sample_uuid('x')` is a helper UDF defined by the repo's data script; use a literal so the statement can be analysed.
    let qsrc = {
        let mut out = String::new();
        let mut rest = qsrc.as_str();
        while let Some(i) = rest.find("sample_uuid(") {
            out.push_str(&rest[..i]);
            let j = rest[i..].find(')').map_or(rest.len(), |k| i + k + 1);
            out.push_str("123e4567-e89b-12d3-a456-426614174000");
            rest = &rest[j..];
        }
        out.push_str(rest);
        out
    };
    let queries: Vec<String> = split(&qsrc).into_iter().filter(|s| matches!(first_word(s).as_str(), "select" | "insert" | "update" | "delete")).collect();
    let info = SchemaInfo::from_ddl(&schema_text);
    let mut analysable: Vec<(String, [bool; 5])> = Vec::new();
    let mut outcome: BTreeMap<&str, usize> = BTreeMap::new();
    for q in &queries {
        let c = compile(&[Input::new("s.cql", schema_text.clone())], &[Input::new("q.cql", format!("-- name: Q :{}\n{q};", if first_word(q) == "select" { "many" } else { "exec" }))]);
        let mut labels = [false; 5];
        let mut blocked = false;
        for d in &c.diagnostics {
            match label_of(d.code) {
                Some(i) => labels[i] = true,
                None if d.is_error() && !matches!(d.code, C::InvalidRestriction) => blocked = true,
                None => {}
            }
        }
        if blocked {
            if std::env::var_os("VERBOSE").is_some() {
                println!("  blocked: {} :: {}", q.lines().next().unwrap_or(""), c.diagnostics.iter().find(|d| d.is_error()).map(|d| d.message.clone()).unwrap_or_default());
            }
            *outcome.entry("cqlc cannot analyse (unsupported or invalid for the accepted schema)").or_default() += 1;
        } else if labels.iter().any(|&b| b) {
            *outcome.entry("analysed, anti-pattern flagged").or_default() += 1;
            analysable.push((q.clone(), labels));
        } else {
            *outcome.entry("analysed, clean").or_default() += 1;
            analysable.push((q.clone(), labels));
        }
    }
    println!("{} statements (SELECT/INSERT/UPDATE/DELETE)", queries.len());
    for (k, v) in &outcome {
        println!("  {v:>3}  {k}");
    }

    if let Some(d) = model_dir {
        let device = Device::flex();
        let (net, _) = load(std::path::Path::new(&d), &device).unwrap();
        let ids: Vec<Vec<u32>> = analysable.iter().map(|(q, _)| encode(q, &info)).collect();
        let seqs: Vec<&[u32]> = ids.iter().map(Vec::as_slice).collect();
        let probs = net.predict(batch_tokens(&seqs, &device)).into_data().try_to_vec::<f32>().unwrap();
        let (mut tp, mut fp, mut fneg, mut exact) = ([0; 5], [0; 5], [0; 5], 0);
        for (i, (_, truth)) in analysable.iter().enumerate() {
            let mut all = true;
            for c in 0..5 {
                let pred = probs[i * LABELS.len() + c] >= 0.5;
                match (pred, truth[c]) {
                    (true, true) => tp[c] += 1,
                    (true, false) => fp[c] += 1,
                    (false, true) => fneg[c] += 1,
                    _ => {}
                }
                all &= pred == truth[c];
            }
            exact += usize::from(all);
        }
        println!("\n== cql-ai vs cqlc on the {} analysable real statements (labels 0-4) ==", analysable.len());
        println!("exact label-set agreement: {}/{}", exact, analysable.len());
        for c in 0..5 {
            println!("  {:<28} tp {:>2}  fp {:>2}  fn {:>2}", LABELS[c], tp[c], fp[c], fneg[c]);
        }
    }
}
