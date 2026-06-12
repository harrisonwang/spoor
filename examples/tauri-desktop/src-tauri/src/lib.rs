use spoor_core::{ParseRequest, parse};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![parse_document])
        .run(tauri::generate_context!())
        .expect("error while running spoor Tauri example");
}

#[cfg(test)]
mod tests {
    use super::parse_document;

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
