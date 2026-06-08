#!/usr/bin/env python3
"""Misc fixture builders: pdf, epub, plain, adversarial."""
import base64
from io import BytesIO
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parent.parent / "fixtures"


# ============================================================
# PDF — using reportlab to make a few simple text-layer PDFs
# ============================================================
def build_pdfs():
    out = ROOT / "pdf"
    out.mkdir(parents=True, exist_ok=True)
    try:
        from reportlab.pdfgen import canvas
        from reportlab.lib.pagesizes import letter
    except ImportError:
        print("reportlab not installed - skipping PDFs")
        return

    # 01: simple single-page PDF
    c = canvas.Canvas(str(out / "01_basic.pdf"), pagesize=letter)
    c.setFont("Helvetica", 14)
    c.drawString(72, 720, "Document title")
    c.setFont("Helvetica", 11)
    c.drawString(72, 690, "First paragraph of the document body.")
    c.drawString(72, 670, "Second paragraph follows.")
    c.showPage()
    c.save()

    # 02: multi-page
    c = canvas.Canvas(str(out / "02_multipage.pdf"), pagesize=letter)
    for i in range(1, 4):
        c.setFont("Helvetica", 12)
        c.drawString(72, 720, f"Page {i} content begins here.")
        c.drawString(72, 700, "Some text on this page.")
        c.showPage()
    c.save()

    # 03: unicode / Chinese (uses default font, may drop chars - intentional;
    # this fixture verifies graceful handling, not perfect output).
    c = canvas.Canvas(str(out / "03_ascii_only.pdf"), pagesize=letter)
    c.setFont("Helvetica", 12)
    c.drawString(72, 720, "ASCII only PDF for baseline test.")
    c.showPage()
    c.save()

    build_image_only_pdf()


def build_image_only_pdf():
    """Build a PDF containing an image object but no text layer."""
    out = ROOT / "pdf"
    out.mkdir(parents=True, exist_ok=True)
    try:
        from reportlab.pdfgen import canvas
        from reportlab.lib.pagesizes import letter
        from reportlab.lib.utils import ImageReader
    except ImportError:
        print("reportlab not installed - skipping image-only PDF")
        return

    png = base64.b64decode(
        "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk"
        "+A8AAQUBAScY42YAAAAASUVORK5CYII="
    )
    c = canvas.Canvas(str(out / "04_image_only.pdf"), pagesize=letter)
    c.drawImage(ImageReader(BytesIO(png)), 72, 620, width=300, height=180)
    c.showPage()
    c.save()


# ============================================================
# EPUB — minimal but correct structure (container + OPF + spine)
# ============================================================
def build_epubs():
    out = ROOT / "epub"
    out.mkdir(parents=True, exist_ok=True)

    # 01: a real epub with two chapters and proper spine ordering
    container = """<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"""
    opf = """<?xml version="1.0"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:title>Test Book</dc:title>
<dc:creator>Test Author</dc:creator>
<dc:identifier id="bookid">urn:test:001</dc:identifier>
<dc:language>en</dc:language>
</metadata>
<manifest>
<item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
<item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
</manifest>
<spine>
<itemref idref="ch1"/>
<itemref idref="ch2"/>
</spine>
</package>"""
    ch1 = """<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><head><title>Chapter 1</title></head><body>
<h1>Chapter One</h1>
<p>This is the first chapter, with a <strong>bold</strong> word.</p>
<ul><li>Item A</li><li>Item B</li></ul>
</body></html>"""
    ch2 = """<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><head><title>Chapter 2</title></head><body>
<h1>Chapter Two</h1>
<p>Second chapter content.</p>
</body></html>"""

    with zipfile.ZipFile(out / "01_basic.epub", "w") as z:
        # mimetype must be the first entry, uncompressed
        z.writestr(zipfile.ZipInfo("mimetype"), "application/epub+zip",
                   compress_type=zipfile.ZIP_STORED)
        z.writestr("META-INF/container.xml", container)
        z.writestr("OEBPS/content.opf", opf)
        z.writestr("OEBPS/ch1.xhtml", ch1)
        z.writestr("OEBPS/ch2.xhtml", ch2)


# ============================================================
# Plain text — encoding variants
# ============================================================
def build_plains():
    out = ROOT / "plain"
    out.mkdir(parents=True, exist_ok=True)
    (out / "01_ascii.txt").write_text("Hello world\nLine two\n", encoding="utf-8")
    (out / "02_utf8.txt").write_text("中文 UTF-8\n日本語\n한글\n", encoding="utf-8")
    (out / "03_gbk.txt").write_bytes("中文 GBK 编码\n第二行\n".encode("gbk"))
    (out / "04_utf16le_bom.txt").write_bytes(
        b"\xff\xfe" + "UTF-16 LE with BOM\nLine 2\n".encode("utf-16-le")
    )
    (out / "05_code.py").write_text(
        "def hello(name):\n    print(f'Hello, {name}')\n\nhello('world')\n",
        encoding="utf-8",
    )


# ============================================================
# Adversarial — broken inputs that should fail cleanly
# ============================================================
def build_adversarial():
    out = ROOT / "adversarial"
    out.mkdir(parents=True, exist_ok=True)
    (out / "01_empty.docx").write_bytes(b"")
    (out / "02_not_zip.docx").write_text("this is not a zip file at all")
    (out / "03_truncated_zip.docx").write_bytes(b"PK\x03\x04" + b"\x00" * 50)
    # Broken JSON
    (out / "04_broken.ipynb").write_text("{not valid json")
    # Bomb-ish: a tiny zip that claims to decompress to gigabytes.
    # Builds a docx where document.xml is highly compressible.
    big_xml = ('<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
               '<w:body><w:p><w:r><w:t>' + ("A" * (5 * 1024 * 1024)) + '</w:t></w:r></w:p></w:body></w:document>')
    types = ('<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
             '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>')
    rels = ('<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
            '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>')
    with zipfile.ZipFile(out / "05_compression_bomb.docx", "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("[Content_Types].xml", types)
        z.writestr("_rels/.rels", rels)
        z.writestr("word/document.xml", big_xml)


if __name__ == "__main__":
    build_pdfs()
    build_epubs()
    build_plains()
    build_adversarial()
    print("done")
