use crate::detect::Format;
use crate::engine::TableFilter;
use crate::json_schema::TableEntry;
use crate::source::Source;
use anyhow::{Result, anyhow};

#[cfg(feature = "tables")]
mod csv;
#[cfg(feature = "office")]
mod docx;
#[cfg(feature = "epub")]
mod epub;
#[cfg(feature = "html")]
mod html;
#[cfg(feature = "notebook")]
mod ipynb;
mod markdown;
#[cfg(feature = "pdf")]
mod pdf;
#[cfg(feature = "pdf")]
#[rustfmt::skip]
mod pdf_engine;
mod plain;
#[cfg(feature = "office")]
mod pptx;
#[cfg(feature = "html")]
mod url;
#[cfg(feature = "tables")]
mod xlsx;
#[cfg(any(feature = "office", feature = "epub"))]
mod xml;

pub fn extract(source: &Source<'_>, format: Format, max_parse_bytes: usize) -> Result<String> {
    match format {
        Format::Url => extract_url(source, max_parse_bytes),
        Format::Html => extract_html(source, max_parse_bytes),
        Format::Markdown => markdown::extract(source),
        Format::Pdf => extract_pdf(source, max_parse_bytes),
        Format::Docx => extract_docx(source, max_parse_bytes),
        Format::Xlsx => extract_xlsx(source, max_parse_bytes),
        Format::Pptx => extract_pptx(source, max_parse_bytes),
        Format::Csv => extract_csv(source, max_parse_bytes),
        Format::Ipynb => extract_ipynb(source, max_parse_bytes),
        Format::Epub => extract_epub(source, max_parse_bytes),
        Format::PlainText => plain::extract(source),
    }
}

pub fn extract_table_entries(
    source: &Source<'_>,
    format: Format,
    source_label: &str,
    filter: &TableFilter,
    max_parse_bytes: usize,
) -> Result<Vec<TableEntry>> {
    match format {
        Format::Csv => extract_csv_tables(source, source_label, filter, max_parse_bytes),
        Format::Xlsx => extract_xlsx_tables(source, source_label, filter, max_parse_bytes),
        _ => Err(anyhow!(
            "--mode json 目前仅支持 csv 和 xlsx；当前格式为 {format}；请用 --mode md 获取 Markdown 输出"
        )),
    }
}

macro_rules! format_extractor {
    ($name:ident, $feature:literal, $module:ident) => {
        #[cfg(feature = $feature)]
        fn $name(source: &Source<'_>, max_parse_bytes: usize) -> Result<String> {
            $module::extract(source, max_parse_bytes)
        }

        #[cfg(not(feature = $feature))]
        fn $name(_source: &Source<'_>, _max_parse_bytes: usize) -> Result<String> {
            Err(anyhow!(concat!(
                "format disabled at compile time; enable feature ",
                $feature
            )))
        }
    };
}

format_extractor!(extract_url, "html", url);
format_extractor!(extract_html, "html", html);
format_extractor!(extract_pdf, "pdf", pdf);
format_extractor!(extract_docx, "office", docx);
format_extractor!(extract_xlsx, "tables", xlsx);
format_extractor!(extract_pptx, "office", pptx);
format_extractor!(extract_csv, "tables", csv);
format_extractor!(extract_ipynb, "notebook", ipynb);
format_extractor!(extract_epub, "epub", epub);

#[cfg(feature = "tables")]
fn extract_csv_tables(
    source: &Source<'_>,
    source_label: &str,
    filter: &TableFilter,
    max_parse_bytes: usize,
) -> Result<Vec<TableEntry>> {
    csv::extract_table_entries(source, source_label, filter, max_parse_bytes)
}

#[cfg(not(feature = "tables"))]
fn extract_csv_tables(
    _source: &Source<'_>,
    _source_label: &str,
    _filter: &TableFilter,
    _max_parse_bytes: usize,
) -> Result<Vec<TableEntry>> {
    Err(anyhow!(
        "CSV support disabled at compile time; enable feature tables"
    ))
}

#[cfg(feature = "tables")]
fn extract_xlsx_tables(
    source: &Source<'_>,
    source_label: &str,
    filter: &TableFilter,
    max_parse_bytes: usize,
) -> Result<Vec<TableEntry>> {
    xlsx::extract_table_entries(source, source_label, filter, max_parse_bytes)
}

#[cfg(not(feature = "tables"))]
fn extract_xlsx_tables(
    _source: &Source<'_>,
    _source_label: &str,
    _filter: &TableFilter,
    _max_parse_bytes: usize,
) -> Result<Vec<TableEntry>> {
    Err(anyhow!(
        "XLSX support disabled at compile time; enable feature tables"
    ))
}
