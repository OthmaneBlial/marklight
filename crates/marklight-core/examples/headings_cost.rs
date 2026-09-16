fn main() {
    for count in [100, 1000, 10000] {
        let source = "# Repeated heading\n\nParagraph.\n\n".repeat(count);
        let start = std::time::Instant::now();
        let doc = marklight_core::Document::parse(&source);
        println!(
            "{} headings: {:.1} ms",
            doc.headings.len(),
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
}
