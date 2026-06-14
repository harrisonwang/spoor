use spoor_core::{ParseRequest, extract_media, parse};

#[tauri::command]
fn parse_document(
    bytes: Vec<u8>,
    source_name: String,
    content_type: Option<String>,
) -> Result<String, String> {
    let mut request = ParseRequest::new(&bytes);
    request.source_name = Some(&source_name);
    request.content_type = content_type.as_deref();

    parse(&request)
        .and_then(|result| {
            serde_json::to_string(&result).map_err(|error| {
                spoor_core::SpoorError::parse_failed(
                    error.to_string(),
                    spoor_core::ParseStage::Render,
                )
            })
        })
        .map_err(|error| error.to_json())
}

// 把解析输出里的 spoor-docx:// 占位符还原成内嵌图片字节。
// 拆出纯逻辑便于单测；command 包装层用 tauri::ipc::Response 直接回二进制，
// 避免把 Vec<u8> 序列化成 JSON 数字数组（前端拿到 ArrayBuffer）。
fn extract_media_bytes(
    bytes: &[u8],
    source_name: &str,
    resource: &str,
    content_type: Option<&str>,
) -> Result<Vec<u8>, String> {
    let mut request = ParseRequest::new(bytes);
    request.source_name = Some(source_name);
    request.content_type = content_type;
    extract_media(&request, resource).map_err(|error| error.to_json())
}

#[tauri::command]
fn extract_document_media(
    bytes: Vec<u8>,
    source_name: String,
    resource: String,
    content_type: Option<String>,
) -> Result<tauri::ipc::Response, String> {
    extract_media_bytes(&bytes, &source_name, &resource, content_type.as_deref())
        .map(tauri::ipc::Response::new)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            parse_document,
            extract_document_media
        ])
        .run(tauri::generate_context!())
        .expect("error while running spoor Tauri example");
}

#[cfg(test)]
mod tests {
    use super::{extract_media_bytes, parse_document};

    #[test]
    fn extracts_embedded_docx_media() {
        let docx = include_bytes!(
            "../../../../crates/spoor-cli/tests/fixtures/docx/16_image_placeholders.docx"
        );
        let image = extract_media_bytes(
            docx,
            "images.docx",
            "spoor-docx://word/media/image1.png",
            None,
        )
        .expect("extract");
        assert_eq!(image, b"first-image");

        let error = extract_media_bytes(docx, "images.docx", "word/media/image1.png", None)
            .expect_err("unsafe uri rejected");
        assert!(error.contains("parse_failed"));
    }

    #[test]
    fn parses_text_through_desktop_command() {
        let output = parse_document(
            "来自 Tauri 的中文文档\n".as_bytes().to_vec(),
            "说明.txt".to_string(),
            Some("text/plain".to_string()),
        )
        .expect("parse");
        let result: serde_json::Value = serde_json::from_str(&output).expect("json");

        assert_eq!(result["stats"]["format"], "text");
        assert_eq!(
            result["content"]["value"]["markdown"],
            "来自 Tauri 的中文文档\n"
        );
    }
}
