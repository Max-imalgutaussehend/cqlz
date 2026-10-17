//! Sequence-length comparison behind the "10x shorter" claim.
//!
//! Measured: UTF-8 bytes, whitespace-separated words and cql-ai ids. Subword (BPE) tokenizers are not
//! run here; they land between "words" and "bytes" (typically ~3-4 bytes/token for code-like text), so
//! bytes is the honest upper bound and words a lower bound for what an LLM tokenizer would produce.

use cql_ai_model::tokenizer::{SchemaInfo, encode};
use cqlz_benchmarks::{SCHEMA, statements};

fn main() {
    let info = SchemaInfo::from_ddl(SCHEMA);
    let stmts = statements();
    let (mut bytes, mut words, mut ids) = (0usize, 0usize, 0usize);
    for s in &stmts {
        bytes += s.len();
        words += s.split_whitespace().count();
        ids += encode(s, &info).len() - 1; // without CLS
    }
    let n = stmts.len() as f64;
    println!("statements        {}", stmts.len());
    println!("bytes/statement   {:.1}", bytes as f64 / n);
    println!("words/statement   {:.1}", words as f64 / n);
    println!("cql-ai ids/stmt   {:.1}", ids as f64 / n);
    println!("bytes / ids       {:.2}x", bytes as f64 / ids as f64);
    println!(
        "bytes/3.5 (BPE est.) / ids  {:.2}x",
        bytes as f64 / 3.5 / ids as f64
    );
    println!("vocabulary        240 ids (vs ~50k for a typical BPE vocabulary)");
}
