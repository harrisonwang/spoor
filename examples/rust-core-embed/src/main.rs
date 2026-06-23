use spoor_core::{ParseRequest, extract_media, parse};

// The function shape is ready to annotate with `#[tauri::command]` in a
// Tauri app. Keeping this example dependency-light makes the core integration
// itself obvious and runnable.
fn parse_for_desktop(bytes: &[u8], source_name: &str) -> Result<String, String> {
    let mut request = ParseRequest::new(bytes);
    request.source_name = Some(source_name);
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

// Resolve one `spoor://docx/part/` placeholder emitted by `parse` into the raw
// embedded image bytes. The desktop host links spoor-core directly, so this
// needs no filesystem access or external process. Annotate with
// `#[tauri::command]` and return the bytes to the frontend as an ArrayBuffer.
fn extract_media_for_desktop(
    bytes: &[u8],
    source_name: &str,
    resource: &str,
) -> Result<Vec<u8>, String> {
    let mut request = ParseRequest::new(bytes);
    request.source_name = Some(source_name);
    extract_media(&request, resource).map_err(|error| error.to_json())
}

fn main() {
    println!(
        "{}",
        parse_for_desktop("来自桌面宿主的中文文档\n".as_bytes(), "说明.txt").expect("parse")
    );

    // 演示按占位符提取内嵌图片字节：解析输出里的 spoor://docx/part/ 占位符可经此还原。
    let docx =
        include_bytes!("../../../crates/spoor-cli/tests/fixtures/docx/16_image_placeholders.docx");
    let image = extract_media_for_desktop(
        docx,
        "images.docx",
        "spoor://docx/part/word/media/image1.png",
    )
    .expect("extract media");
    println!(
        "从 spoor://docx/part/word/media/image1.png 提取到 {} 字节",
        image.len()
    );
}

#[cfg(test)]
mod tests {
    use super::extract_media_for_desktop;

    #[test]
    fn extracts_embedded_docx_media() {
        let docx = include_bytes!(
            "../../../crates/spoor-cli/tests/fixtures/docx/16_image_placeholders.docx"
        );
        let image = extract_media_for_desktop(
            docx,
            "images.docx",
            "spoor://docx/part/word/media/image1.png",
        )
        .expect("extract");
        assert_eq!(image, b"first-image");

        let error = extract_media_for_desktop(docx, "images.docx", "word/media/image1.png")
            .expect_err("unsafe uri rejected");
        assert!(error.contains("parse_failed"));
    }
}
