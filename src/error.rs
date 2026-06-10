use serde::Serialize;
use std::fmt;

const IMAGE_ONLY_PDF_REASON: &str = "纯图片 PDF（无文本层）";
const IMAGE_ONLY_PDF_HINT: &str = "该 PDF 没有文本层，需要 OCR，但 pith 不执行 OCR。";
const PARSE_MEMORY_LIMIT_REASON: &str = "超出解析预算";
const UNSUPPORTED_FORMAT_REASON: &str = "无法识别的格式";
const UNSUPPORTED_FORMAT_HINT: &str = "无法识别或不支持该输入的格式。请用 --format 显式指定格式。";

/// Stable, machine-readable error code. Serialized as snake_case (e.g.
/// `image_only_pdf`). Consumers should branch on `code`, **not** on the
/// human-readable `reason`/`hint`, which are localized and may be reworded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ImageOnlyPdf,
    ParseBudgetExceeded,
    UnsupportedFormat,
}

/// A machine-readable extraction failure for agents and other CLI consumers.
///
/// `code` is the stable contract (English snake_case); `reason`/`hint` are
/// the human-facing message (Chinese) and must not be branched on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StructuredError {
    pub is_error: bool,
    pub code: ErrorCode,
    pub reason: String,
    pub hint: String,
    pub recoverable: bool,
}

impl StructuredError {
    pub fn image_only_pdf() -> Self {
        Self {
            is_error: true,
            code: ErrorCode::ImageOnlyPdf,
            reason: IMAGE_ONLY_PDF_REASON.to_string(),
            hint: IMAGE_ONLY_PDF_HINT.to_string(),
            recoverable: true,
        }
    }

    pub fn parse_memory_limit(max_bytes: usize, stage: &str) -> Self {
        Self {
            is_error: true,
            code: ErrorCode::ParseBudgetExceeded,
            reason: PARSE_MEMORY_LIMIT_REASON.to_string(),
            hint: format!(
                "解析在 {stage} 阶段超出了 {max_bytes} 字节的数据量预算。请缩小输入范围，或用 --max-parse-bytes <n> 调高上限。"
            ),
            recoverable: true,
        }
    }

    pub fn unsupported_format() -> Self {
        Self {
            is_error: true,
            code: ErrorCode::UnsupportedFormat,
            reason: UNSUPPORTED_FORMAT_REASON.to_string(),
            hint: UNSUPPORTED_FORMAT_HINT.to_string(),
            recoverable: true,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize structured error")
    }
}

impl fmt::Display for StructuredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}

impl std::error::Error for StructuredError {}
