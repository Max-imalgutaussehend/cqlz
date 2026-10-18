//! Real-world check: the KillrVideo reference schemas and query examples (Apache-2.0) through `cqlc-core`.
//!
//!     cargo run --release --bin realworld
//!
//! Reports how much of the real files `cqlc` accepts as published. Column `DEFAULT` values are *not* CQL (Cassandra's
//! own grammar has none), so the 4.0/5.0 schema files of that sample are expected to fail on them.

use cql_ai_model::tokenizer::statements;
use cqlc_core::{Input, compile};
use std::collections::BTreeMap;

fn split(src: &str) -> Vec<String> {
    statements(src)
        .into_iter()
        .map(|s| src[s.range].to_string())
        .collect()
}

fn first_word(s: &str) -> String {
    s.split(|c: char| !c.is_ascii_alphabetic())
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn main() {
    let dir = "data/real";
    println!("== schemas through cqlc-core ==");
    for v in ["v3", "v4", "v5"] {
        let src = std::fs::read_to_string(format!("{dir}/kv-schema-{v}.cql")).unwrap();
        let stmts: Vec<String> = split(&src)
            .into_iter()
            .filter(|s| matches!(first_word(s).as_str(), "create" | "alter"))
            .collect();
        let mut ok = 0;
        let mut why: BTreeMap<String, usize> = BTreeMap::new();
        let mut good = Vec::new();
        for t in &stmts {
            let c = compile(&[Input::new("s.cql", format!("{t};"))], &[]);
            // a statement that only fails because it refers to a table defined elsewhere still parsed
            match c
                .diagnostics
                .iter()
                .find(|d| d.is_error() && d.code.as_str() == "CQL001")
            {
                None => {
                    ok += 1;
                    good.push(format!("{t};"));
                }
                Some(d) => *why.entry(d.message.chars().take(60).collect()).or_default() += 1,
            }
        }
        println!("schema {v}: {ok}/{} statements parsed", stmts.len());
        for (m, n) in why.iter().take(4) {
            println!("    rejected x{n}: {m}");
        }
        let _ = good;
    }

    queries("v4", true);
    queries("v5", false);
}

/// Removes `DEFAULT <literal-or-call>` so the rest of a table definition can be analysed.
fn strip_default(src: &str) -> String {
    let (mut out, mut rest) = (String::new(), src);
    while let Some(i) = rest.find(" DEFAULT ") {
        out.push_str(&rest[..i]);
        let mut j = i + 9;
        let b = rest.as_bytes();
        while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
            j += 1;
        }
        if j < b.len() && b[j] == b'(' {
            j = rest[j..].find(')').map_or(rest.len(), |k| j + k + 1);
        }
        rest = &rest[j..];
    }
    out.push_str(rest);
    out
}

fn queries(v: &str, strip: bool) {
    println!(
        "\n== query examples ({v}) against the {v} schema{} ==",
        if strip {
            " with DEFAULT clauses removed"
        } else {
            ""
        }
    );
    let raw = std::fs::read_to_string(format!("data/real/kv-schema-{v}.cql"))
        .unwrap()
        .replace("killrvideo.", "");
    let schema_text = if strip { strip_default(&raw) } else { raw };
    let qsrc = std::fs::read_to_string(format!("data/real/kv-schema-{v}-query-examples.cql"))
        .unwrap()
        .replace("killrvideo.", "");
    // `sample_uuid('x')` is a helper UDF of the sample's data script; use a literal so the statement can be analysed.
    let qsrc = {
        let (mut out, mut rest) = (String::new(), qsrc.as_str());
        while let Some(i) = rest.find("sample_uuid(") {
            out.push_str(&rest[..i]);
            let j = rest[i..].find(')').map_or(rest.len(), |k| i + k + 1);
            out.push_str("123e4567-e89b-12d3-a456-426614174000");
            rest = &rest[j..];
        }
        out.push_str(rest);
        out
    };
    let queries: Vec<String> = split(&qsrc)
        .into_iter()
        .filter(|s| {
            matches!(
                first_word(s).as_str(),
                "select" | "insert" | "update" | "delete"
            )
        })
        .collect();
    let mut outcome: BTreeMap<String, usize> = BTreeMap::new();
    for q in &queries {
        let kind = if first_word(q) == "select" {
            "many"
        } else {
            "exec"
        };
        let c = compile(
            &[Input::new("s.cql", schema_text.clone())],
            &[Input::new("q.cql", format!("-- name: Q :{kind}\n{q};"))],
        );
        let why = c
            .diagnostics
            .iter()
            .find(|d| d.is_error() && !matches!(d.code.as_str(), "CQL111" | "CQL112"));
        *outcome
            .entry(why.map_or("analysed".into(), |d| {
                format!(
                    "{}: {}",
                    d.code.as_str(),
                    d.message.chars().take(70).collect::<String>()
                )
            }))
            .or_default() += 1;
    }
    println!("{} statements (SELECT/INSERT/UPDATE/DELETE)", queries.len());
    for (k, n) in &outcome {
        println!("  {n:>3}  {k}");
    }
}
