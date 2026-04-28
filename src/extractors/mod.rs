use crate::format::Format;
use crate::source::Source;
use anyhow::Result;

mod url;
mod html;
mod markdown;
mod pdf;
mod docx;
mod xlsx;
mod pptx;
mod csv;
mod ipynb;
mod epub;
mod plain;

pub fn extract(source: &Source, format: Format) -> Result<String> {
    match format {
        Format::Url      => url::extract(source),
        Format::Html     => html::extract(source),
        Format::Markdown => markdown::extract(source),
        Format::Pdf      => pdf::extract(source),
        Format::Docx     => docx::extract(source),
        Format::Xlsx     => xlsx::extract(source),
        Format::Pptx     => pptx::extract(source),
        Format::Csv      => csv::extract(source),
        Format::Ipynb    => ipynb::extract(source),
        Format::Epub     => epub::extract(source),
        Format::PlainText => plain::extract(source),
    }
}
