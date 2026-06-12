//! Shared quick-xml helpers.

use quick_xml::events::BytesStart;

/// Find an attribute by local name (ignoring any namespace prefix) and return
/// its value as an owned UTF-8 string. Malformed attributes are skipped.
pub fn attr(e: &BytesStart, local_name: &[u8]) -> Option<String> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == local_name)
        .and_then(|a| String::from_utf8(a.value.into_owned()).ok())
}
