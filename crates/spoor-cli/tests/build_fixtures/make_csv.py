#!/usr/bin/env python3
"""
CSV fixtures — focus on encoding detection and delimiter sniffing,
which is where naive `cat` falls down for Chinese users.
"""
from pathlib import Path
import csv

OUT = Path(__file__).resolve().parent.parent / "fixtures" / "csv"
OUT.mkdir(parents=True, exist_ok=True)


# ---------- 01: simple comma ----------
def build_01_basic():
    p = OUT / "01_basic.csv"
    with p.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["Name", "Score", "Note"])
        w.writerow(["Alice", "95", "first"])
        w.writerow(["Bob", "80", "second"])
        w.writerow(["Carol", "70", "third"])


# ---------- 02: tab-separated (.tsv style content but in .csv extension) ----------
def build_02_tab_separated():
    p = OUT / "02_tab_separated.csv"
    with p.open("w", newline="") as f:
        w = csv.writer(f, delimiter="\t")
        w.writerow(["a", "b", "c"])
        w.writerow(["1", "2", "3"])


# ---------- 03: semicolon (European Excel default) ----------
def build_03_semicolon():
    p = OUT / "03_semicolon.csv"
    p.write_text("col1;col2;col3\nfoo;bar;baz\n", encoding="utf-8")


# ---------- 04: GBK encoded (China) ----------
def build_04_gbk():
    p = OUT / "04_gbk.csv"
    text = "姓名,分数,备注\n张三,95,优秀\n李四,80,良好\n王五,70,中等\n"
    p.write_bytes(text.encode("gbk"))


# ---------- 05: UTF-8 with BOM (Excel default on Windows) ----------
def build_05_utf8_bom():
    p = OUT / "05_utf8_bom.csv"
    text = "name,score\nAlice,95\nBob,80\n"
    p.write_bytes("\ufeff".encode("utf-8") + text.encode("utf-8"))


# ---------- 06: quoted fields with embedded commas, newlines, quotes ----------
def build_06_quoted():
    p = OUT / "06_quoted.csv"
    with p.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["name", "description"])
        w.writerow(["Alice", "has, commas in it"])
        w.writerow(["Bob", 'has "quotes" inside'])
        w.writerow(["Carol", "has\nnewline"])


# ---------- 07: empty file ----------
def build_07_empty():
    (OUT / "07_empty.csv").write_text("")


# ---------- 08: pipe-separated (some data exports) ----------
def build_08_pipe():
    (OUT / "08_pipe.csv").write_text("a|b|c\n1|2|3\nx|y|z\n")


# ---------- 09: ragged rows (different column counts) ----------
def build_09_ragged():
    (OUT / "09_ragged.csv").write_text("a,b,c\n1,2\nx,y,z,w\n")


# ---------- 10: large file (truncation test - 2000 rows) ----------
def build_10_large():
    p = OUT / "10_large.csv"
    with p.open("w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["id", "value"])
        for i in range(2000):
            w.writerow([i, f"row_{i}"])


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("build_") and callable(fn):
            print(f"Building {name}...")
            fn()
