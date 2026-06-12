use spoor_core::{Format, ParseRequest, parse_document};
use std::time::Instant;

fn main() {
    let bytes = include_bytes!("../../spoor-cli/tests/fixtures/plain/01_ascii.txt");
    let iterations = 10_000;
    let started = Instant::now();
    for _ in 0..iterations {
        let mut request = ParseRequest::new(bytes);
        request.format_hint = Some(Format::PlainText);
        parse_document(&request).expect("parse fixture");
    }
    let elapsed = started.elapsed();
    println!(
        "warm core parse: {iterations} iterations in {elapsed:?} ({:?}/call)",
        elapsed / iterations
    );
}
