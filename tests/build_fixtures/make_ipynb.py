#!/usr/bin/env python3
"""IPYNB fixtures."""
from pathlib import Path
import json

OUT = Path(__file__).resolve().parent.parent / "fixtures" / "ipynb"
OUT.mkdir(parents=True, exist_ok=True)


def write(name, data):
    (OUT / name).write_text(json.dumps(data, ensure_ascii=False, indent=1))


# ---------- 01: typical mix of markdown + code + outputs (outputs ignored) ----------
def build_01_basic():
    write("01_basic.ipynb", {
        "cells": [
            {"cell_type": "markdown", "metadata": {},
             "source": ["# Notebook title\n", "Some prose.\n"]},
            {"cell_type": "code", "execution_count": 1, "metadata": {},
             "outputs": [
                 {"name": "stdout", "output_type": "stream",
                  "text": ["should not appear in output\n"]},
                 {"data": {"text/plain": ["42"]}, "execution_count": 1,
                  "metadata": {}, "output_type": "execute_result"},
             ],
             "source": ["print('hello')\n", "42"]},
            {"cell_type": "markdown", "metadata": {},
             "source": "Trailing markdown."},
        ],
        "metadata": {"kernelspec": {"name": "python3",
                                    "language": "python",
                                    "display_name": "Python 3"}},
        "nbformat": 4, "nbformat_minor": 5,
    })


# ---------- 02: source as string vs source as list — must handle both ----------
def build_02_source_formats():
    write("02_source_formats.ipynb", {
        "cells": [
            {"cell_type": "markdown", "metadata": {},
             "source": "single string source"},
            {"cell_type": "markdown", "metadata": {},
             "source": ["array\n", "of\n", "lines"]},
            {"cell_type": "code", "metadata": {},
             "outputs": [], "execution_count": None,
             "source": ""},  # empty string
            {"cell_type": "code", "metadata": {},
             "outputs": [], "execution_count": None,
             "source": []},  # empty array
        ],
        "metadata": {}, "nbformat": 4, "nbformat_minor": 5,
    })


# ---------- 03: language hint from kernelspec ----------
def build_03_language_hint():
    write("03_language_hint.ipynb", {
        "cells": [
            {"cell_type": "code", "metadata": {},
             "outputs": [], "execution_count": 1,
             "source": "x <- 1:10\nplot(x)"},
        ],
        "metadata": {"kernelspec": {"name": "ir",
                                    "language": "R",
                                    "display_name": "R"}},
        "nbformat": 4, "nbformat_minor": 5,
    })


# ---------- 04: raw cells (skipped) ----------
def build_04_raw_cells():
    write("04_raw_cells.ipynb", {
        "cells": [
            {"cell_type": "markdown", "metadata": {}, "source": "# Title"},
            {"cell_type": "raw", "metadata": {}, "source": "raw content should be skipped"},
            {"cell_type": "markdown", "metadata": {}, "source": "After raw."},
        ],
        "metadata": {}, "nbformat": 4, "nbformat_minor": 5,
    })


# ---------- 05: malformed (missing cells array) ----------
def build_05_malformed():
    write("05_malformed.ipynb", {"metadata": {}, "nbformat": 4})


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("build_") and callable(fn):
            print(f"Building {name}...")
            fn()
