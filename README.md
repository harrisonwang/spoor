# gist

> Convert files and URLs to LLM-friendly markdown — one binary, no external deps.

```
$ gist https://example.com         # web page → markdown
$ gist report.docx                  # Word → markdown
$ gist data.xlsx                    # Excel → markdown table
$ gist slides.pptx                  # PowerPoint → markdown
$ gist paper.pdf                    # PDF (text layer) → markdown
$ gist notebook.ipynb               # Jupyter → markdown
$ gist book.epub                    # ePub → markdown
$ gist data.csv                     # CSV → markdown table
$ cat foo.docx | gist               # stdin support (planned)
$ gist file.docx --json             # structured output for pipelines
```

## Supported formats

| Format        | Status   | Library                       |
| ------------- | -------- | ----------------------------- |
| HTML / URL    | skeleton | `scraper` + readability (TODO)|
| Markdown      | done     | (passthrough)                 |
| PDF (text)    | done     | `pdf-extract`                 |
| docx          | skeleton | `zip` + `quick-xml` (custom)  |
| xlsx          | done     | `calamine`                    |
| pptx          | skeleton | `zip` + `quick-xml` (custom)  |
| CSV / TSV     | done     | `csv` + `chardetng`           |
| ipynb         | done     | `serde_json`                  |
| epub          | skeleton | `zip` + spine ordering (TODO) |
| Plain text    | done     | `chardetng` for encoding      |

PDF OCR, audio, video, and old `.doc/.xls/.ppt` are **out of scope** by design.
This tool stays as a single static binary; if you need OCR or media, pipe
through dedicated tools.

## Build

```
cargo build --release
./target/release/gist file.docx
```

## Roadmap

- [ ] readability-style main content extraction for HTML/URL
- [ ] Full docx: tables, footnotes, lists with proper nesting
- [ ] Full pptx: tables, speaker notes
- [ ] Full epub: spine-ordered chapters with markdown fidelity
- [ ] Protocol-aware URL dispatch (YouTube subtitles, GitHub repos, arXiv PDFs)
- [ ] `--max-bytes`, `--head N` truncation flags
- [ ] stdin support
