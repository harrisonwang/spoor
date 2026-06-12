use spoor_core::{ParseRequest, parse};

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

fn main() {
    println!(
        "{}",
        parse_for_desktop(b"hello from a desktop host\n", "note.txt").expect("parse")
    );
}
