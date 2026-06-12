use serde::{Deserialize, Serialize};
use std::fmt;

const IMAGE_ONLY_PDF_REASON: &str = "纯图片 PDF（无文本层）";
const IMAGE_ONLY_PDF_HINT: &str = "该 PDF 没有文本层，需要 OCR，但 spoor 不执行 OCR。";
const PARSE_MEMORY_LIMIT_REASON: &str = "超出解析预算";
const UNSUPPORTED_FORMAT_REASON: &str = "无法识别的格式";
const UNSUPPORTED_FORMAT_HINT: &str = "无法识别或不支持该输入的格式。请显式指定 format。";
const ENCRYPTED_PDF_REASON: &str = "受密码保护的 PDF";
const ENCRYPTED_PDF_HINT: &str =
    "该 PDF 受密码保护，spoor 不支持解密。请先解除密码保护，再重新运行。";
const LEGACY_OR_ENCRYPTED_OFFICE_REASON: &str = "旧版或加密的 Office 文档";
const LEGACY_OR_ENCRYPTED_OFFICE_HINT: &str = "该文件是 OLE/CFB 容器：可能是受密码保护的 Office 文档，也可能是旧版二进制格式（.doc/.xls/.ppt）。spoor 都不支持：加密文件请先解除密码，旧版格式请先另存为 docx/xlsx/pptx。";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ImageOnlyPdf,
    ParseBudgetExceeded,
    UnsupportedFormat,
    EncryptedPdf,
    LegacyOrEncryptedOffice,
    InvalidContainer,
    ParseFailed,
}

impl ErrorCode {
    pub const ALL: [ErrorCode; 7] = [
        ErrorCode::ImageOnlyPdf,
        ErrorCode::ParseBudgetExceeded,
        ErrorCode::UnsupportedFormat,
        ErrorCode::EncryptedPdf,
        ErrorCode::LegacyOrEncryptedOffice,
        ErrorCode::InvalidContainer,
        ErrorCode::ParseFailed,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::ImageOnlyPdf => "image_only_pdf",
            ErrorCode::ParseBudgetExceeded => "parse_budget_exceeded",
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
    pub fn image_only_pdf() -> Self {
        Self::new(
            ErrorCode::ImageOnlyPdf,
            IMAGE_ONLY_PDF_REASON,
            IMAGE_ONLY_PDF_HINT,
            true,
            ParseStage::Parse,
        )
    }

    pub fn parse_memory_limit(max_bytes: usize, stage: &str) -> Self {
        Self::new(
            ErrorCode::ParseBudgetExceeded,
            PARSE_MEMORY_LIMIT_REASON,
            format!(
                "解析在 {stage} 阶段超出了 {max_bytes} 字节的数据量预算。请缩小输入范围，或调高 max_parse_bytes（CLI: --max-parse-bytes）。"
            ),
            true,
            ParseStage::Limits,
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
            format!("无效的 {label} 容器"),
            format!(
                "文件不是有效的 {label} 容器（可能为空、损坏或扩展名与内容不符）。请确认文件完整；若扩展名不可靠，显式指定真实格式。"
            ),
            true,
            ParseStage::Parse,
        )
    }

    pub fn parse_failed(reason: impl Into<String>, stage: ParseStage) -> Self {
        Self::new(
            ErrorCode::ParseFailed,
            reason,
            "输入未能完成解析。请确认文件完整、格式正确且未超过资源限制。",
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
