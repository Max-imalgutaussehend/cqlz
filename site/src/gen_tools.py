#!/usr/bin/env python3
"""Writes src/cqlint.html, src/cql-migrate.html, src/cqlprof.html (graphics first, little text). Run before partials.py."""
import pathlib
from gen_benchmarks import panel, tile, key, BL, OR, GR, LT, donut

HERE = pathlib.Path(__file__).parent

def page(title, desc, name, hl, logo, alt, lead, links, install, body):
    btns = "".join(f'<a class="btn{" primary" if i == 0 else ""}" href="{u}">{t}</a>' for i, (t, u) in enumerate(links))
    return f"""{title}
---
{desc}
---
<div class="hero left"><div class="split">
<img class="logo" src="logos/{logo}.png" alt="{alt}">
<div>
<h1>{name}<b>{hl}</b></h1>
<p class="lead">{lead}</p>
<div class="btns">{btns}</div>
<div class="install"><i>$</i> {install}</div>
</div></div></div>
{body}
"""

def sect(eye, h, inner): return f'<section><div class="wrap"><div class="eyebrow">{eye}</div><h2>{h}</h2>{inner}</div></section>'
def chips(items): return '<div class="key" style="gap:8px 10px;margin-top:10px">' + "".join(f'<span class="badge {c}">{t}</span>' for t, c in items) + "</div>"
def pre(t): return f"<pre><code>{t}</code></pre>"
def grid(*p): return '<div class="gfx">' + "".join(p) + "</div>"
def tiles(*t): return '<div class="tiles">' + "".join(t) + "</div>"

# ---------------- cqlint ----------------
before = "select id,name from ks.users where id = ? and name &gt; 'a' order by name desc limit 10;"
after = "<span class=k>SELECT</span> id, name\n<span class=k>FROM</span> ks.users\n<span class=k>WHERE</span> id = ?\n  <span class=k>AND</span> name &gt; 'a'\n<span class=k>ORDER BY</span> name <span class=k>DESC</span>\n<span class=k>LIMIT</span> 10;"
rules = [("allow-filtering", "lose"), ("select-star", "lose"), ("unbounded-select", "lose"), ("count-scan", "lose"), ("large-in", "lose"), ("large-batch", "lose"), ("native-secondary-index", "lose"), ("lightweight-transaction", "tie"), ("destructive", "lose"), ("non-idempotent-ddl (fixable)", "win"), ("list-column", "tie"), ("null-write", "lose"), ("wide-table", "lose")]
cqlint = page(
    "cqlint: a very fast linter and formatter for CQL",
    "cqlint formats CQL and finds performance and safety problems (ALLOW FILTERING, tombstone writes, unbounded scans) in Cassandra and ScyllaDB projects.",
    "cql", "int", "cqlint", "cqlint logo: the Rust crab with a check mark badge",
    "Format CQL and catch the statements that hurt at scale, in milliseconds. Built on the lexer of cqlc.",
    [("GitHub", "https://github.com/Max-imalgutaussehend/cqlint"), ("crates.io", "https://crates.io/crates/cqlint")], "cargo install cqlint",
    sect("Format", "One layout, safe by construction", grid(
        panel("Before", "any style", f"<pre><code>{before}</code></pre>"),
        panel("After", "keywords, clauses, wrapping, comments kept", f"<pre><code>{after}</code></pre>"),
        panel("Safety net", "tokens and comments must be unchanged", donut(1, "100 %", "idempotent", BL), '<div class="txt">Checked on real KillrVideo and cqlc files; if the token stream would change, the file is left alone.</div>'),
    ))
    + sect("Lint", "13 rules for CQL that does not scale", grid(
        panel("Rules", "red = warning, white = note, blue = fixable", chips(rules)),
        panel("Type-aware", "<code>--schema schema/</code> adds cqlc-core checks", '<div class="diagram"><div class="node"><b>statement</b>from your .cql</div><div class="arrow">→</div><div class="node"><b>schema</b>unknown column, type mismatch, missing partition key</div></div>'),
    ))
    + sect("Speed", "3,600 files in a quarter of a second", tiles(tile("0.24 s", "win", "lint, 3,600 files, 23 MB"), tile("0.53 s", "win", "format check, same files"), tile("330k", "tie", "statements per second, lint"), tile("8 cores", "tie", "Apple M3, rayon")))
    + sect("CI", "Three lines", pre('<span class=c># .github/workflows</span>\n- run: cargo install cqlint\n- run: cqlint format --check .\n- run: cqlint check --output-format github .')),
)

# ---------------- cql-migrate ----------------
flow = '<div class="diagram">' + "".join(f'<div class="node"><b>{a}</b>{b}</div>' + ('<div class="arrow">→</div>' if i < 4 else "") for i, (a, b) in enumerate([("new", "0007_add_email.cql"), ("verify", "replay offline"), ("diff", "schema/ vs migrations"), ("up", "lock, apply, record"), ("drift", "cluster vs migrations")])) + "</div>"
states = [("applied", "win"), ("pending", "tie"), ("CHANGED", "lose"), ("FAILED", "lose"), ("RUNNING", "lose"), ("MISSING", "lose")]
migrate = page(
    "cql-migrate: schema migrations for Cassandra and ScyllaDB",
    "cql-migrate applies ordered CQL migrations with checksums and a cluster-wide lock, checks the chain offline, generates diffs and detects schema drift.",
    "cql-", "migrate", "cql-migrate", "cql-migrate logo: the Rust crab with an up-arrow badge",
    "Ordered, checksummed, locked schema migrations. Offline chain checks, schema diff and live drift detection.",
    [("GitHub", "https://github.com/Max-imalgutaussehend/cql-migrate"), ("crates.io", "https://crates.io/crates/cql-migrate")], "cargo install cql-migrate",
    sect("Workflow", "From file to cluster and back", flow + grid(
        panel("State of every migration", "status, with exit code 1 on problems", chips(states)),
        panel("Tested on", "full lifecycle in CI", chips([("ScyllaDB", "win"), ("Cassandra 5.0", "win"), ("Cassandra 4.1 in CI", "tie")]), '<div class="txt">Apply, checksum protection, failure and resolve, lock, drift.</div>'),
    ))
    + sect("Safe by design", "What it protects you from", tiles(tile("SHA-256", "win", "over tokens: comments and format do not matter"), tile("LWT lock", "win", "one runner at a time, TTL"), tile("agreement", "win", "waits after every DDL"), tile("no rollback", "tie", "DDL cannot be undone; failures are recorded")))
    + sect("In CI", "Keep schema files and migrations in sync", pre('cql-migrate verify                 <span class=c># replay all migrations with cqlc-core</span>\ncql-migrate diff schema/           <span class=c># exit 1 if schema/ and migrations/ disagree</span>\ncql-migrate up                     <span class=c># lock, apply, record</span>\ncql-migrate drift                  <span class=c># did somebody ALTER production by hand?</span>')),
)

# ---------------- cqlprof ----------------
sample = "<span class=k>SELECT</span> * <span class=k>FROM</span> pf.q <span class=k>WHERE</span> pk = 1\n  client latency   p50 1.11 ms   p99 1.95 ms\n  trace (scylla)   server 833 µs, 1 replica targeted\n  work             10 live row(s), <span class=k>2990 tombstone(s)</span>\n  warning [PROF001] scanned 2990 tombstones to return 10 live rows"
findings = [("PROF001 tombstone-scan", "lose"), ("PROF002 many-sstables", "lose"), ("PROF003 large-read", "lose"), ("PROF004 slow-statement", "lose"), ("PROF005 tail-latency", "lose"), ("PROF006 lightweight-transaction", "tie")]
prof = page(
    "cqlprof: EXPLAIN for CQL",
    "cqlprof runs SELECT statements with tracing on Cassandra and ScyllaDB and reports tombstones, sstables, replicas, slow steps and latency percentiles.",
    "cql", "prof", "cqlprof", "cqlprof logo: the Rust crab with a stopwatch badge",
    "Cassandra has no EXPLAIN, but it has tracing. cqlprof reads the trace and says where the time and the work went.",
    [("GitHub", "https://github.com/Max-imalgutaussehend/cqlprof"), ("crates.io", "https://crates.io/crates/cqlprof")], "cargo install cqlprof",
    sect("Output", "A trace, explained", grid(
        panel("Example", "5,000 deleted rows in one partition", f"<pre><code>{sample}</code></pre>", wide=True),
        panel("Findings", "from the trace, not from guessing", chips(findings)),
        panel("Engines", "trace parsing tested on real traces", chips([("ScyllaDB", "win"), ("Cassandra 5.0", "win"), ("Cassandra 4.x", "tie")])),
    ))
    + sect("Safe to run", "Bounded and read-only by default", tiles(tile("SELECT", "win", "only reads run; writes need --allow-writes"), tile("1 page", "win", "a missing LIMIT cannot scan the table"), tile("sampled", "tie", "? markers get values from a real row"), tile("p50 / p99", "tie", "untraced repetitions, first run dropped")))
    + sect("Use", "One command", pre('cqlprof -e "SELECT * FROM shop.orders WHERE customer = 1"\ncqlprof queries/*.cql -k shop --repeat 100 --format json')),
)

if __name__ == "__main__":
    (HERE / "cqlint.html").write_text(cqlint)
    (HERE / "cql-migrate.html").write_text(migrate)
    (HERE / "cqlprof.html").write_text(prof)
