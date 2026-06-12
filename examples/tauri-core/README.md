# Tauri core integration shape

`parse_for_desktop` has the same argument and result shape as a Tauri command,
without adding Tauri itself as a dependency to this example.

```bash
cargo run -p spoor-tauri-core-example
```

In an application, annotate the function with `#[tauri::command]` and pass
file bytes from the frontend.
