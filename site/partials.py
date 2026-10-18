#!/usr/bin/env python3
"""Builds the static pages from small templates so header/footer stay identical.  python3 site/partials.py"""
import pathlib

HERE = pathlib.Path(__file__).parent
PAGES = [
    ("cqlc.html", "cqlc"),
    ("cqlx.html", "cqlx"),
    ("cql-ai.html", "cql-ai"),
    ("benchmarks.html", "Benchmarks"),
    ("examples.html", "Examples"),
    ("research.html", "Research (alpha)"),
]
ALL = [("index.html", "Family")] + PAGES

def page(slug, title, desc, body):
    nav = "".join(
        f'<a class="link" href="{href}"{" aria-current=\"page\"" if href == slug else ""}>{label}</a>'
        for href, label in PAGES
    )
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<meta name="description" content="{desc}">
<meta name="theme-color" content="#171010">
<link rel="icon" href="logo.png">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&display=swap">
<link rel="stylesheet" href="style.css">
</head>
<body>
<div class="topbar"><b>NEW</b> &nbsp;Real-world KillrVideo results, YCSB against cqlx and a learned schema designer &rarr; <a href="benchmarks.html">see the benchmarks</a></div>
<header class="nav"><div class="wrap">
<a class="brand" href="index.html">cql<b>z</b></a>{nav}
<span class="spacer"></span>
<a class="chip" href="https://github.com/Max-imalgutaussehend/cqlz">GitHub</a>
<a class="chip cta" href="cqlc.html">Get started</a>
</div></header>
<main>
{body}
</main>
<footer><div class="wrap">
<span>The cqlz family: <a href="cqlc.html">cqlc</a> · <a href="cqlx.html">cqlx</a> · <a href="cql-ai.html">cql-ai</a> · <a href="research.html">research</a>. Apache-2.0.</span>
<span>Built by <a href="https://github.com/Max-imalgutaussehend">Max-imalgutaussehend</a></span>
</div></footer>
</body>
</html>
"""

def main():
    for slug, _ in ALL:
        src = HERE / "src" / slug
        title, desc, body = src.read_text().split("\n---\n", 2)
        (HERE / slug).write_text(page(slug, title.strip(), desc.strip(), body))
        print("wrote", slug)

if __name__ == "__main__":
    main()
