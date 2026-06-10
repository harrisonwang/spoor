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

pub fn extract(source: &Source, format: Format, max_parse_bytes: usize) -> Result<String> {
    match format {
        Format::Url => url::extract(source, max_parse_bytes),
        Format::Html => html::extract(source, max_parse_bytes),
        Format::Markdown => markdown::extract(source),
        Format::Pdf => pdf::extract(source, max_parse_bytes),
        Format::Docx => docx::extract(source, max_parse_bytes),
        Format::Xlsx => xlsx::extract(source, max_parse_bytes),
        Format::Pptx => pptx::extract(source, max_parse_bytes),
        Format::Csv => csv::extract(source, max_parse_bytes),
        Format::Ipynb => ipynb::extract(source, max_parse_bytes),
        Format::Epub => epub::extract(source, max_parse_bytes),
        Format::PlainText => plain::extract(source),
    }
}

pub fn extract_table_entries(
    source: &Source,
    format: Format,
    source_label: &str,
    filter: &TableFilter,
    max_parse_bytes: usize,
) -> Result<Vec<TableEntry>> {
    match format {
        Format::Csv => csv::extract_table_entries(source, source_label, filter, max_parse_bytes),
        Format::Xlsx => xlsx::extract_table_entries(source, source_label, filter, max_parse_bytes),
        _ => Err(anyhow!(
            "--mode json 目前仅支持 csv 和 xlsx；当前格式为 {format}；请用 --mode md 获取 Markdown 输出"
        )),
    }
}
