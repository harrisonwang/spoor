use crate::extract::TableFilter;
use crate::format::Format;
use crate::json_schema::TableEntry;
use crate::source::Source;
use anyhow::{Result, anyhow};

mod csv;
mod docx;
mod epub;
mod html;
mod ipynb;
mod markdown;
mod pdf;
mod plain;
mod pptx;
mod url;
mod xlsx;
mod xml;

pub fn extract(source: &Source, format: Format) -> Result<String> {
    match format {
        Format::Url => url::extract(source),
        Format::Html => html::extract(source),
        Format::Markdown => markdown::extract(source),
        Format::Pdf => pdf::extract(source),
        Format::Docx => docx::extract(source),
        Format::Xlsx => xlsx::extract(source),
        Format::Pptx => pptx::extract(source),
        Format::Csv => csv::extract(source),
        Format::Ipynb => ipynb::extract(source),
        Format::Epub => epub::extract(source),
        Format::PlainText => plain::extract(source),
    }
}

pub fn extract_table_entries(
    source: &Source,
    format: Format,
    source_label: &str,
    filter: &TableFilter,
) -> Result<Vec<TableEntry>> {
    match format {
        Format::Csv => csv::extract_table_entries(source, source_label, filter),
        Format::Xlsx => xlsx::extract_table_entries(source, source_label, filter),
        _ => Err(anyhow!(
            "--mode json currently supports csv and xlsx only; got {format}; use --mode md for Markdown output"
        )),
    }
}
