//! Public API for converting files and URLs into LLM-friendly text.

mod error;
mod extract;
mod extractors;
mod format;
mod json_schema;
mod limits;
mod output;
mod render;
mod source;

pub use error::{
    IMAGE_ONLY_PDF_HINT, IMAGE_ONLY_PDF_REASON, PARSE_MEMORY_LIMIT_REASON, StructuredError,
};
pub use extract::{
    ExtractOptions, ExtractedDocument, ExtractedTables, ResolvedInput, TableFilter, extract_md,
    extract_table_entries, resolve_input,
};
pub use format::{Format, FormatArg};
pub use json_schema::{
    HeaderInfo, JsonOutput, PreambleInfo, RowRange, TABLE_SCHEMA_VERSION, TABLE_USAGE, TableEntry,
    a1_range, cells_to_values,
};
pub use limits::{DEFAULT_MAX_PARSE_BYTES, MIN_MAX_PARSE_BYTES};
pub use render::{
    DEFAULT_MAX_OUTPUT_BYTES, LimitedOutput, MIN_MAX_OUTPUT_BYTES, OutputMode, default_mode_for,
    limit_markdown_output, render_documents, render_json, render_json_limited,
};
pub use source::{Source, SourceInput, is_url};
