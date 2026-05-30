//! Public API for converting files and URLs into LLM-friendly text.

mod extract;
mod extractors;
mod format;
mod json_schema;
mod limits;
mod output;
mod render;
mod source;

pub use extract::{
    ExtractOptions, ExtractedDocument, ResolvedInput, TableFilter, extract_md,
    extract_table_entries, resolve_input,
};
pub use format::{Format, FormatArg};
pub use json_schema::{
    HeaderInfo, JsonOutput, PreambleInfo, RowRange, TABLE_SCHEMA_VERSION, TABLE_USAGE, TableEntry,
    a1_range, cells_to_values,
};
pub use render::{OutputMode, default_mode_for, render_documents, render_json};
pub use source::{Source, SourceInput, is_url};
