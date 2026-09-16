use marklight_core::Document;
use marklight_render::{HtmlOptions, TerminalOptions, render_html, render_terminal};
use std::{fs, hint::black_box, path::Path, time::Instant};
fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts");
    fs::create_dir_all(&root).unwrap();
    let chunk = "# Repeated section\n\nA **readable** paragraph with a [link](#repeated-section), words and Unicode café 日本語.\n\n- A list item\n- Another item\n\n| Feature | Status |\n|---|---|\n| Offline | Yes |\n\n```rust\nfn main() { println!(\"Marklight\"); }\n```\n\n";
    let warm = Document::parse(chunk);
    black_box(render_html(&warm, &HtmlOptions::default()));
    let mut results = Vec::new();
    for size in [10_000, 100_000, 1_000_000, 5_000_000, 10_000_000] {
        let mut source = chunk.repeat(size / chunk.len() + 1);
        let mut end = size;
        while !source.is_char_boundary(end) {
            end -= 1;
        }
        source.truncate(end);
        fs::write(root.join(format!("large-{size}.md")), &source).unwrap();
        let start = Instant::now();
        let doc = Document::parse(black_box(&source));
        let parse = ms(start);
        let start = Instant::now();
        let terminal = render_terminal(
            &doc,
            TerminalOptions {
                ansi: false,
                ..Default::default()
            },
        );
        let terminal_ms = ms(start);
        black_box(&terminal);
        let start = Instant::now();
        let html = render_html(&doc, &HtmlOptions::default());
        let html_ms = ms(start);
        black_box(&html);
        println!(
            "{} bytes: parse {:.2} ms, terminal {:.2} ms, HTML {:.2} ms",
            source.len(),
            parse,
            terminal_ms,
            html_ms
        );
        results.push(serde_json::json!({"bytes":source.len(),"parse_ms":parse,"terminal_ms":terminal_ms,"html_ms":html_ms,"headings":doc.headings.len(),"html_bytes":html.len()}));
    }
    fs::write(
        root.join("benchmark.json"),
        serde_json::to_vec_pretty(&results).unwrap(),
    )
    .unwrap();
}
