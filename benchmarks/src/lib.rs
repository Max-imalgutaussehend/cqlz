//! Shared fixtures: the `shop` example from cqlc (schema + 3 query files).

pub const SCHEMA: &str = include_str!("../data/schema.cql");
pub const QUERIES: &str = include_str!("../data/queries.cql");

/// Query statements stripped of `-- name:` annotations, one per element (no trailing `;`).
pub fn statements() -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for line in QUERIES.lines() {
        if line.trim_start().starts_with("--") {
            continue;
        }
        cur.push_str(line);
        cur.push('\n');
        if line.trim_end().ends_with(';') {
            out.push(cur.trim().trim_end_matches(';').to_string());
            cur.clear();
        }
    }
    out
}
