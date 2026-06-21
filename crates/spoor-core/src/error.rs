use serde::{Deserialize, Serialize};
use std::fmt;

const PDF_NO_EXTRACTABLE_CONTENT_REASON: &str = "PDF 无可提取内容";
const PDF_NO_EXTRACTABLE_CONTENT_HINT: &str = "使用 VLM 处理。";
const PARSE_MEMORY_LIMIT_REASON: &str = "超出解析上限";
const UNSUPPORTED_FORMAT_REASON: &str = "无法识别的格式";
const UNSUPPORTED_FORMAT_HINT: &str = "当前不支持该格式。";
const ENCRYPTED_PDF_REASON: &str = "加密的 PDF";
const ENCRYPTED_PDF_HINT: &str = "去除密码后重试。";
const LEGACY_OR_ENCRYPTED_OFFICE_REASON: &str = "旧版或加密 Office 格式";
const LEGACY_OR_ENCRYPTED_OFFICE_HINT: &str =
    "若已加密，去除密码；旧版格式（.doc/.xls/.ppt）需另存为 docx/xlsx/pptx。";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    PdfNoExtractableContent,
    ParseBudgetExceeded,
    WorkBudgetExceeded,
    UnsupportedFormat,
    EncryptedPdf,
    LegacyOrEncryptedOffice,
    InvalidContainer,
    ParseFailed,
}

impl ErrorCode {
    pub const ALL: [ErrorCode; 8] = [
        ErrorCode::PdfNoExtractableContent,
        ErrorCode::ParseBudgetExceeded,
        ErrorCode::WorkBudgetExceeded,
        ErrorCode::UnsupportedFormat,
        ErrorCode::EncryptedPdf,
        ErrorCode::LegacyOrEncryptedOffice,
        ErrorCode::InvalidContainer,
        ErrorCode::ParseFailed,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::PdfNoExtractableContent => "pdf_no_extractable_content",
            ErrorCode::ParseBudgetExceeded => "parse_budget_exceeded",
            ErrorCode::WorkBudgetExceeded => "work_budget_exceeded",
            ErrorCode::UnsupportedFormat => "unsupported_format",
            ErrorCode::EncryptedPdf => "encrypted_pdf",
            ErrorCode::LegacyOrEncryptedOffice => "legacy_or_encrypted_office",
            ErrorCode::InvalidContainer => "invalid_container",
            ErrorCode::ParseFailed => "parse_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseStage {
    Detect,
    Read,
    Parse,
    Limits,
    Render,
}

/// Stable error contract shared by Rust, CLI, Python, Node, and WASM.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpoorError {
    pub is_error: bool,
    pub code: ErrorCode,
    pub reason: String,
    pub hint: String,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<ParseStage>,
}

/// Compatibility alias for callers migrating from the original CLI library.
pub type StructuredError = SpoorError;

impl SpoorError {
    pub fn pdf_no_extractable_content() -> Self {
        Self::new(
            ErrorCode::PdfNoExtractableContent,
            PDF_NO_EXTRACTABLE_CONTENT_REASON,
            PDF_NO_EXTRACTABLE_CONTENT_HINT,
            true,
            ParseStage::Parse,
        )
    }

    pub fn parse_memory_limit(max_bytes: usize, stage: &str) -> Self {
        Self::new(
            ErrorCode::ParseBudgetExceeded,
            PARSE_MEMORY_LIMIT_REASON,
            format!(
                "解析在 {stage} 阶段超过 {max_bytes} 字节上限。可缩减输入范围，或调高 --max-parse-bytes。"
            ),
            true,
            ParseStage::Limits,
        )
    }

    pub fn work_budget_exceeded() -> Self {
        Self::new(
            ErrorCode::WorkBudgetExceeded,
            "超出运算量上限",
            "调高 --max-work-units；处理不可信文件时配合进程隔离与超时。",
            true,
            ParseStage::Parse,
        )
    }

    pub fn unsupported_format() -> Self {
        Self::new(
            ErrorCode::UnsupportedFormat,
            UNSUPPORTED_FORMAT_REASON,
            UNSUPPORTED_FORMAT_HINT,
            true,
            ParseStage::Detect,
        )
    }

    pub fn encrypted_pdf() -> Self {
        Self::new(
            ErrorCode::EncryptedPdf,
            ENCRYPTED_PDF_REASON,
            ENCRYPTED_PDF_HINT,
            false,
            ParseStage::Parse,
        )
    }

    pub fn legacy_or_encrypted_office() -> Self {
        Self::new(
            ErrorCode::LegacyOrEncryptedOffice,
            LEGACY_OR_ENCRYPTED_OFFICE_REASON,
            LEGACY_OR_ENCRYPTED_OFFICE_HINT,
            false,
            ParseStage::Detect,
        )
    }

    pub fn invalid_container(label: &str) -> Self {
        Self::new(
            ErrorCode::InvalidContainer,
            format!("无效的 {label} 文件"),
            format!("确认文件完整；若扩展名不匹配，手动指定格式。"),
            true,
            ParseStage::Parse,
        )
    }

    pub fn parse_failed(reason: impl Into<String>, stage: ParseStage) -> Self {
        Self::new(
            ErrorCode::ParseFailed,
            reason,
            "确认文件完整、格式正确，且未超出资源上限。",
            true,
            stage,
        )
    }

    pub(crate) fn from_anyhow(error: anyhow::Error, stage: ParseStage) -> Self {
        error
            .downcast_ref::<SpoorError>()
            .cloned()
            .unwrap_or_else(|| Self::parse_failed(error.root_cause().to_string(), stage))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("serialize structured error")
    }

    fn new(
        code: ErrorCode,
        reason: impl Into<String>,
        hint: impl Into<String>,
        recoverable: bool,
        stage: ParseStage,
    ) -> Self {
        Self {
            is_error: true,
            code,
            reason: reason.into(),
            hint: hint.into(),
            recoverable,
            stage: Some(stage),
        }
    }
}

impl fmt::Display for SpoorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}

impl std::error::Error for SpoorError {}

#[cfg(test)]
mod tests {
    use super::ErrorCode;

    #[test]
    fn as_str_matches_serde_serialization() {
        for code in ErrorCode::ALL {
            let wire = serde_json::to_string(&code).expect("serialize code");
            assert_eq!(wire, format!("\"{}\"", code.as_str()));
        }
    }
}
