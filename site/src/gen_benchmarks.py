#!/usr/bin/env python3
"""Writes src/benchmarks.html: inline-SVG charts from the measured numbers. Run before partials.py."""
import math, pathlib
HERE = pathlib.Path(__file__).parent
BL, OR, GR, LT = "var(--blue)", "var(--orange)", "#6b5b57", "#c9bcb6"

def arc(cx, cy, r, a0, a1):
    x0, y0 = cx + r * math.sin(a0), cy - r * math.cos(a0)
    x1, y1 = cx + r * math.sin(a1), cy - r * math.cos(a1)
    return f"M{x0:.1f},{y0:.1f} A{r},{r} 0 {1 if a1 - a0 > math.pi else 0} 1 {x1:.1f},{y1:.1f}"

def donut(frac, big, sub, color=BL, size=150):
    r = 52; c = size / 2
    d = (f'<circle cx="{c}" cy="{c}" r="{r}" stroke="{color}" stroke-width="16" fill="none"/>' if frac >= 1 else
         f'<path d="{arc(c, c, r, 0, 2 * math.pi * frac)}" stroke="{color}" stroke-width="16" fill="none"/>' if frac > 0 else "")
    return (f'<svg viewBox="0 0 {size} {size}" style="max-width:170px;margin:auto" role="img" aria-label="{big} {sub}"><circle cx="{c}" cy="{c}" r="{r}" stroke="{GR}" stroke-opacity=".35" stroke-width="16" fill="none"/>{d}'
            f'<text x="{c}" y="{c + 2}" text-anchor="middle" class="big">{big}</text><text x="{c}" y="{c + 20}" text-anchor="middle" class="m">{sub}</text></svg>')

def pie(slices, size=230):
    tot = sum(v for _, v, _ in slices); a = 0; out = ""; cx = cy = size / 2; r = size / 2 - 6
    for name, v, col in slices:
        b = a + 2 * math.pi * v / tot
        x0, y0, x1, y1 = cx + r * math.sin(a), cy - r * math.cos(a), cx + r * math.sin(b), cy - r * math.cos(b)
        out += f'<path d="M{cx},{cy} L{x0:.1f},{y0:.1f} A{r},{r} 0 {1 if b - a > math.pi else 0} 1 {x1:.1f},{y1:.1f} Z" fill="{col}" stroke="var(--bg2)" stroke-width="2"/>'
        a = b
    return f'<svg viewBox="0 0 {size} {size}" style="max-width:230px;margin:auto">{out}</svg>'

def key(items): return '<div class="key">' + "".join(f'<span><i style="background:{c}"></i>{n}</span>' for n, c in items) + "</div>"

def vbars(groups, series, ymax, fmt="{:.0f}", h=190, ref=None):
    """groups: [(label, [v per series])]; series: [(name, color)]"""
    W = 420; pl, pb, pt = 34, 26, 12; ph = h - pb - pt; gw = (W - pl) / len(groups); bw = gw * .7 / len(series)
    o = ""
    for t in range(0, 5):
        y = pt + ph - ph * t / 4; o += f'<line class="grid" x1="{pl}" x2="{W}" y1="{y}" y2="{y}"/><text class="m" x="{pl - 6}" y="{y + 4}" text-anchor="end">{fmt.format(ymax * t / 4)}</text>'
    for gi, (lab, vals) in enumerate(groups):
        x0 = pl + gi * gw + gw * .15
        for si, v in enumerate(vals):
            bh = ph * v / ymax; x = x0 + si * bw
            o += f'<rect x="{x:.1f}" y="{pt + ph - bh:.1f}" width="{bw - 3:.1f}" height="{bh:.1f}" fill="{series[si][1]}" rx="2"/>'
            o += f'<text x="{x + (bw - 3) / 2:.1f}" y="{pt + ph - bh - 4:.1f}" text-anchor="middle" class="m" style="font-size:10px">{fmt.format(v)}</text>'
        o += f'<text x="{pl + gi * gw + gw / 2:.1f}" y="{h - 8}" text-anchor="middle">{lab}</text>'
    if ref is not None:
        y = pt + ph - ph * ref / ymax; o += f'<line x1="{pl}" x2="{W}" y1="{y}" y2="{y}" stroke="{OR}" stroke-dasharray="4 3"/>'
    return f'<svg viewBox="0 0 {W} {h}">{o}</svg>'

def radar(axes, series, size=340):
    cx = cy = size / 2; R = size / 2 - 70; n = len(axes); o = ""
    pt = lambda i, v: (cx + R * v * math.sin(2 * math.pi * i / n), cy - R * v * math.cos(2 * math.pi * i / n))
    for lv in (.25, .5, .75, 1):
        o += '<polygon class="grid" points="' + " ".join("%.1f,%.1f" % pt(i, lv) for i in range(n)) + '"/>'
    for i, a in enumerate(axes):
        x, y = pt(i, 1); o += f'<line class="grid" x1="{cx}" y1="{cy}" x2="{x:.1f}" y2="{y:.1f}"/>'
        lx, ly = pt(i, 1.22); o += f'<text x="{lx:.1f}" y="{ly + 4:.1f}" text-anchor="middle">{a}</text>'
    for name, col, vals in series:
        o += f'<polygon points="' + " ".join("%.1f,%.1f" % pt(i, v) for i, v in enumerate(vals)) + f'" fill="{col}" fill-opacity=".18" stroke="{col}" stroke-width="2"/>'
    return f'<svg viewBox="0 0 {size} {size}" style="max-width:300px;margin:auto">{o}</svg>'

def lollipop(rows, vmax, unit="%", w=420):
    rh = 34; h = rh * len(rows) + 8; lx = 150; o = ""
    for i, (lab, v, col) in enumerate(rows):
        y = 20 + i * rh; x = lx + (w - lx - 60) * v / vmax
        o += f'<text x="{lx - 10}" y="{y + 4}" text-anchor="end">{lab}</text><line x1="{lx}" x2="{x:.1f}" y1="{y}" y2="{y}" stroke="{col}" stroke-width="3"/><circle cx="{x:.1f}" cy="{y}" r="7" fill="{col}"/>'
        o += f'<text x="{x + 14:.1f}" y="{y + 4}" style="font-weight:600">{v:g}{unit}</text>'
    return f'<svg viewBox="0 0 {w} {h}">{o}</svg>'

def line(points_sets, xs, ymax, h=190):
    W = 420; pl, pb, pt = 40, 26, 12; ph = h - pb - pt; pw = W - pl - 14; o = ""
    for t in range(5):
        y = pt + ph - ph * t / 4; o += f'<line class="grid" x1="{pl}" x2="{W - 14}" y1="{y}" y2="{y}"/><text class="m" x="{pl - 6}" y="{y + 4}" text-anchor="end">{ymax * t / 4:g}M</text>'
    X = lambda i: pl + pw * i / (len(xs) - 1)
    for i, x in enumerate(xs): o += f'<text x="{X(i):.1f}" y="{h - 8}" text-anchor="middle">{x}</text>'
    for name, col, vals, dash in points_sets:
        pts = " ".join(f"{X(i):.1f},{pt + ph - ph * v / ymax:.1f}" for i, v in enumerate(vals))
        o += f'<polyline points="{pts}" fill="none" stroke="{col}" stroke-width="3" {"stroke-dasharray=\"5 4\"" if dash else ""}/>'
        if not dash:
            for i, v in enumerate(vals): o += f'<circle cx="{X(i):.1f}" cy="{pt + ph - ph * v / ymax:.1f}" r="4" fill="{col}"/>'
    return f'<svg viewBox="0 0 {W} {h}">{o}</svg>'

def versus(a, b, unit, ratio, wins=True):
    """two big numbers with a ratio badge"""
    return (f'<svg viewBox="0 0 420 120"><text x="10" y="50" class="big" style="font-size:40px;fill:{BL if wins else OR}">{a}</text><text x="10" y="74" class="m">{unit[0]}</text>'
            f'<text x="410" y="50" text-anchor="end" class="big" style="font-size:40px;fill:{LT}">{b}</text><text x="410" y="74" text-anchor="end" class="m">{unit[1]}</text>'
            f'<rect x="150" y="30" width="120" height="30" rx="15" fill="{BL if wins else OR}"/><text x="210" y="50" text-anchor="middle" style="fill:#0c1a22;font-weight:700;font-size:15px">{ratio}</text></svg>')

def panel(h, one, svg, extra="", wide=False): return f'<div class="panel{" wide" if wide else ""}"><h4>{h}</h4><p class="one">{one}</p>{svg}{extra}</div>'
def tile(n, cls, t, b=""): return f'<div class="tile"><div class="n {cls}">{n}</div><div class="t">{t}</div></div>'
def sect(eye, h, tiles, panels, details=""): return f'<section><div class="wrap"><div class="eyebrow">{eye}</div><h2>{h}</h2>{tiles}<div class="gfx">{"".join(panels)}</div>{details}</div></section>'
def det(title, body): return f'<details><summary>{title}</summary><div class="in">{body}</div></details>'

KEYS_WIN = key([("better", BL), ("baseline", GR), ("equal", LT), ("worse", OR)])
head = """cqlz benchmarks: cqlc, cqlx, cql-ai and cqlz-gym measured
---
Measured results for the cqlz family as charts: cqlc vs sqlparser, cqlx vs the raw driver and charybdis, cql-ai vs keyword rules, cqlz-gym vs an exact ILP.
---
<div class="hero left">
<h1>Bench<b>marks</b></h1>
<p class="lead">Measured, including the results that do not flatter us. Apple M3, one node in Docker.</p>
<div class="legend"><span><i style="background:var(--blue)"></i>better</span><span><i style="background:#c9bcb6"></i>equal</span><span><i style="background:var(--orange)"></i>worse</span><span><i style="background:#6b5b57"></i>baseline</span></div>
</div>
"""
tiles = '<div class="tiles">' + "".join([
    tile("3.8×", "win", "cqlc parse speed vs sqlparser"),
    tile("40/40", "win", "cqlc, Cassandra 5.0 schema"),
    tile("97 %", "tie", "cqlx vs raw driver"),
    tile("+0.29", "win", "cql-ai F1 vs keyword rules"),
    tile("+8.2 %", "win", "gym design, real ScyllaDB"),
    tile("400×", "lose", "cql-ai slower than cqlc"),
]) + "</div>"
glance = f'<section class="first"><div class="wrap"><div class="eyebrow">At a glance</div><h2>Six numbers</h2>{tiles}</div></section>'

cqlc = sect("cqlc", "Fast, and complete for Cassandra 5", "", [
    panel("Parse 22 statements", "3.8× faster than sqlparser", versus("33 µs", "126 µs", ("cqlc", "sqlparser"), "3.8×"), '<div class="txt">sqlparser is a SQL parser: no CQL schema checks.</div>'),
    panel("KillrVideo schemas accepted", "Cassandra 5.0 fully parsed",
          '<div style="display:grid;grid-template-columns:repeat(3,1fr);gap:0">' + donut(1, "14/14", "v3") + donut(16 / 23, "16/23", "v4", OR) + donut(1, "40/40", "v5") + '</div>',
          '<div class="txt">v4: the 7 rejected use <code>DEFAULT</code>, which is not CQL.</div>'),
    panel("Real query examples analysed", "v4 and v5 sample queries",
          '<div style="display:grid;grid-template-columns:repeat(2,1fr);gap:4px">' + donut(18 / 20, "18/20", "v4") + donut(27 / 35, "27/35", "v5") + '</div>',
          '<div class="txt">The rest are invalid CQL in the samples (wrong vector size, <code>date - date</code>).</div>'),
], det("Numbers", "<table><tr><th>cqlc 1.1</th><th class='num'>Time</th></tr><tr><td>lexer, 36 statements</td><td class='num'>13.9 µs</td></tr><tr><td>parser</td><td class='num'>52 µs</td></tr><tr><td>full analysis (lex, parse, resolve, type-check)</td><td class='num'>103 µs</td></tr></table>"))

ycsb = vbars([("A  50/50", [95.2, 98.6, 100.1]), ("B  95/5", [113.4, 114.5, 107.2]), ("C  100/0", [110.8, 116.5, 117.6])], [("cqlx", LT), ("raw driver", GR), ("charybdis", GR)], 140, "{:.0f}")
cqlx = sect("cqlx", "Level with the raw driver; safe by construction", "", [
    panel("YCSB throughput, k ops/s", "all within run-to-run noise", ycsb, key([("cqlx", LT), ("raw driver", GR)]) , wide=False),
    panel("cqlx as share of raw driver", "95 to 97 %", donut(.97, "97 %", "median", LT), '<div class="txt">Zipfian 0.99, 64 tasks, median of 3 runs.</div>'),
    panel("Build one statement", "cqlx pays 250 ns for compile-time checks", versus("320 ns", "68 ns", ("cqlx, validated", "format!, unchecked"), "+250 ns", wins=False), '<div class="txt">Hidden by a ~1 ms network round trip.</div>'),
], det("Not measured here", "cqlx 1.0 partial updates, LWT, batches and streaming are covered by tests on ScyllaDB and Cassandra 5.0/4.1, not by this benchmark."))

radar_svg = radar(["unseen worlds", "unseen families", "other engine"],
                  [("keyword rules", GR, [.557, .624, .516]), ("cql-ai, statement only", LT, [.683, .674, .624]), ("cql-ai, with data stats", BL, [.855, .889, .802])])
fams = [("NoSQLBench tabular", 1169 + 434 + 432, BL), ("NoSQLBench iot", 680, "#4aa3d1"), ("cassandra-stress", 583 + 24, OR), ("KillrVideo comments", 541, "#b3470f"), ("NoSQLBench queue (tombstones)", 460, LT), ("TSBS cpu", 410, "#8a7b76"), ("NoSQLBench keyvalue", 336, "#e8a07a"), ("YCSB", 215, "#5d7f92")]
pie_svg = pie([(n, v, c) for n, v, c in fams])
cqlai = sect("cql-ai", "Predicts measured bottlenecks, on data it never saw", "", [
    panel("Macro-F1 on held-out data", "blue wins on all three", radar_svg, key([("keyword rules", GR), ("statement only", LT), ("with data stats", BL)])),
    panel("Training data: 5,284 measured statements", "ScyllaDB 2,670 + Cassandra 5.0 2,614", pie_svg, key([(n, c) for n, _, c in fams])),
    panel("Time to check 36 statements", "cqlc is the faster, simpler tool; cql-ai answers a different question", versus("0.11 ms", "41.7 ms", ("cqlc analysis", "cql-ai forward pass"), "400×", wins=False), '<div class="txt">42 ms is far inside a CI budget.</div>'),
], det("Numbers", "<table><tr><th>Macro-F1</th><th class='num'>with data stats</th><th class='num'>statement only</th><th class='num'>keyword rules</th></tr><tr><td>unseen worlds</td><td class='num'>0.855</td><td class='num'>0.683</td><td class='num'>0.557</td></tr><tr><td>unseen families</td><td class='num'>0.889</td><td class='num'>0.674</td><td class='num'>0.624</td></tr><tr><td>other engine (Cassandra 5.0)</td><td class='num'>0.802</td><td class='num'>0.624</td><td class='num'>0.516</td></tr></table><p class='sub'>Trained on ScyllaDB only for this table. Without data statistics the model is only a little better than the rules: the gain comes from knowing your data size. Released weights are trained on both engines.</p>"))

gym_l = lollipop([("greedy", 11.7, OR), ("exact B&amp;B, 15 s", 7.8, OR), ("one table per query", 4.4, GR), ("learned policy", .6, LT), ("ILP", 0, BL)], 12)
real = vbars([("sensors", [8.2]), ("shop", [1.4]), ("killrvideo", [0])], [("best design", BL)], 10, "+{:.1f}%", h=180)
scale = line([("measured", BL, [.383, .738, 1.40, 2.04], False), ("ideal", GR, [.383, .766, 1.53, 3.06], True)], [1, 2, 4, 8], 3)
gym = sect("cqlz-gym", "An exact ILP wins; the design pays off on a real database", "", [
    panel("Extra cost over the best design", "80–150 queries, lower is better", gym_l, '<div class="txt">The learned policy is a fast approximation, never better than the ILP.</div>'),
    panel("Instances solved to optimality in 15 s", "80–150 queries", '<div style="display:grid;grid-template-columns:repeat(2,1fr);gap:4px">' + donut(1, "8/8", "ILP") + donut(0, "0/8", "B&amp;B", OR) + '</div>', '<div class="txt">ILP time: under 10 ms.</div>'),
    panel("Real ScyllaDB: throughput vs one table per query", "+8.2 % on sensors, equal on killrvideo", real, '<div class="txt">Noise is a few percent; the model predicted −23 % cost for sensors, measured −7.6 % latency.</div>'),
    panel("Environment steps per second, M steps/s", "91 % efficient on 4 threads; 8 threads use the efficiency cores", scale, key([("measured", BL), ("ideal", GR)])),
], det("Table: cost relative to best, by workload size", "<table><tr><th>Queries</th><th class='num'>greedy</th><th class='num'>B&amp;B</th><th class='num'>ILP</th><th class='num'>learned</th></tr><tr><td>4–7</td><td class='num'>1.095</td><td class='num'>1.000</td><td class='num'>1.000</td><td class='num'>1.000</td></tr><tr><td>8–16</td><td class='num'>1.009</td><td class='num'>1.000</td><td class='num'>1.000</td><td class='num'>1.003</td></tr><tr><td>25–50</td><td class='num'>1.054</td><td class='num'>1.001</td><td class='num'>1.000</td><td class='num'>1.005</td></tr><tr><td>80–150</td><td class='num'>1.117</td><td class='num'>1.078</td><td class='num'>1.000</td><td class='num'>1.006</td></tr></table>"))

repro = """<section><div class="wrap"><div class="eyebrow">Reproduce</div><h2>Run it yourself</h2>
<pre><code>git clone https://github.com/Max-imalgutaussehend/cqlz &amp;&amp; cd cqlz/benchmarks
cargo bench --bench frontend        <span class="c"># cqlc vs sqlparser</span>
cargo bench --bench inference       <span class="c"># cql-ai vs cqlc</span>
cargo run --release --bin realworld <span class="c"># KillrVideo coverage</span>
cargo run --release --bin ycsb -- 50000 10   <span class="c"># cqlx vs raw driver (Docker)</span>

git clone https://github.com/Max-imalgutaussehend/cqlz-lab &amp;&amp; cd cqlz-lab
cargo run --release -p cqlz-gym -- eval --policy artifacts --instances 8 --exact-seconds 15
cargo run --release -p cqlz-gym -- bench</code></pre>
<details><summary>Data sources and what is not measured</summary><div class="in"><p class="sub">KillrVideo schemas and queries (Apache-2.0), YCSB core workloads A/B/C, NoSQLBench (cql-keyvalue2, cql-tabular2, cql-iot, cql-timeseries2), cassandra-stress profiles, TSBS CPU table, DataStax data-modelling examples (shapes of the gym workloads). Not measured: cluster throughput, multi-node scaling, tail latency under overload, one machine only.</p></div></details>
</div></section>"""
(HERE / "benchmarks.html").write_text(head + "\n" + glance + "\n" + cqlc + "\n" + cqlx + "\n" + cqlai + "\n" + gym + "\n" + repro + "\n")
