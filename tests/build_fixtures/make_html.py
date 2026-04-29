#!/usr/bin/env python3
"""HTML fixtures - articles, ad-heavy pages, encoding."""
from pathlib import Path

OUT = Path(__file__).resolve().parent.parent / "fixtures" / "html"
OUT.mkdir(parents=True, exist_ok=True)


# ---------- 01: clean article with semantic markup ----------
def build_01_article():
    (OUT / "01_article.html").write_text("""<!doctype html>
<html><head>
<title>The article title</title>
<meta charset="utf-8">
</head><body>
<header><nav>Home About Contact</nav></header>
<article>
<h1>The article title</h1>
<p>The opening paragraph contains the lead and is written by the author.</p>
<h2>First section</h2>
<p>This is the body of the first section, with a <a href="https://example.com">link</a>
and some <strong>bold</strong> and <em>emphasized</em> text.</p>
<ul>
<li>First bullet</li>
<li>Second bullet</li>
</ul>
<h2>Second section</h2>
<p>Another paragraph.</p>
</article>
<footer>Copyright 2025</footer>
</body></html>""", encoding="utf-8")


# ---------- 02: ad-heavy / cluttered — readability should isolate main content ----------
def build_02_cluttered():
    (OUT / "02_cluttered.html").write_text("""<!doctype html>
<html><head><title>News</title></head><body>
<div class="ad-banner">SUBSCRIBE NOW! 50% OFF! BUY BUY BUY!</div>
<nav>Home | Politics | Sports | Opinion | Subscribe</nav>
<aside class="related">Related: 10 Things You Won't Believe</aside>
<main>
<article>
<h1>Real news headline</h1>
<p>The actual story content begins here. This is the meat of the article that
a user actually wants to read.</p>
<p>A second paragraph of substantive content.</p>
</article>
</main>
<aside class="newsletter">Sign up for our newsletter!</aside>
<footer>© 2025 Example News, Inc.</footer>
</body></html>""", encoding="utf-8")


# ---------- 03: tables in HTML ----------
def build_03_table():
    (OUT / "03_table.html").write_text("""<!doctype html>
<html><body>
<h1>Data table</h1>
<table>
<thead><tr><th>Name</th><th>Age</th></tr></thead>
<tbody>
<tr><td>Alice</td><td>30</td></tr>
<tr><td>Bob</td><td>25</td></tr>
</tbody>
</table>
</body></html>""", encoding="utf-8")


# ---------- 04: GBK encoding (no <meta charset>, must detect) ----------
def build_04_gbk_no_meta():
    text = """<html><body>
<h1>中文标题</h1>
<p>这是一段中文正文,测试编码自动检测。</p>
</body></html>"""
    (OUT / "04_gbk_no_meta.html").write_bytes(text.encode("gbk"))


# ---------- 05: script and style tags should be stripped ----------
def build_05_scripts_styles():
    (OUT / "05_scripts_styles.html").write_text("""<!doctype html>
<html><head>
<style>body { color: red; } .ad { display: none; }</style>
<script>alert('this should not appear in output');</script>
</head><body>
<h1>Real content</h1>
<p>This is what the user wants to see.</p>
<script>tracking.send('view');</script>
</body></html>""", encoding="utf-8")


# ---------- 06: links with title and rel attributes ----------
def build_06_links():
    (OUT / "06_links.html").write_text("""<!doctype html>
<html><body>
<p>Visit <a href="https://example.com" title="Example">our site</a> for details.</p>
<p>Or read the <a href="https://docs.example.com/guide">guide</a>.</p>
</body></html>""", encoding="utf-8")


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("build_") and callable(fn):
            print(f"Building {name}...")
            fn()
