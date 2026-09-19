use marklight_core::Document;
use pulldown_cmark::{BlockQuoteKind, Event, Tag, TagEnd};
use textwrap::core::display_width;

#[derive(Debug, Clone, Copy)]
pub struct TerminalOptions {
    pub width: usize,
    pub ansi: bool,
    pub dark: bool,
}

impl Default for TerminalOptions {
    fn default() -> Self {
        Self {
            width: 80,
            ansi: true,
            dark: true,
        }
    }
}

pub fn render_terminal(document: &Document, options: TerminalOptions) -> String {
    let mut r = Renderer {
        options: TerminalOptions {
            width: options.width.max(20),
            ..options
        },
        output: String::new(),
        buffer: String::new(),
        lists: vec![],
        marker: None,
        quotes: 0,
        styles: vec![],
        heading: None,
        in_code: false,
        code_index: 0,
        links: vec![],
        table: None,
    };
    for event in &document.events {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                r.flush(true);
                r.in_code = true;
                let block = &document.code_blocks[r.code_index];
                r.code_index += 1;
                let language = if block.language.is_empty() {
                    "code"
                } else {
                    &block.language
                };
                r.line(&format!("╭─ {}", safe(language)));
                for line in safe(&block.code).lines() {
                    let available = r
                        .options
                        .width
                        .saturating_sub(display_width(&r.prefix()) + 2)
                        .max(1);
                    for wrapped in textwrap::wrap(line, available) {
                        r.line(&format!("│ {wrapped}"));
                    }
                }
                r.line("╰─");
                r.blank();
            }
            Event::End(TagEnd::CodeBlock) => r.in_code = false,
            _ if r.in_code => {}
            Event::Start(Tag::Paragraph) => r.flush(false),
            Event::End(TagEnd::Paragraph) => r.flush(r.lists.is_empty()),
            Event::Start(Tag::Heading { level, .. }) => {
                r.flush(true);
                r.heading = Some(*level as u8);
                r.start_style(if *level as u8 <= 2 { "1;36" } else { "1" });
            }
            Event::End(TagEnd::Heading(_)) => {
                r.end_style();
                let underline = r.heading == Some(1);
                let length = display_width(&r.buffer)
                    .min(r.options.width.saturating_sub(display_width(&r.prefix())));
                r.flush(false);
                if underline {
                    r.line(&"━".repeat(length));
                }
                r.heading = None;
                r.blank();
            }
            Event::Start(Tag::BlockQuote(kind)) => {
                r.flush(true);
                r.quotes += 1;
                if let Some(kind) = kind {
                    r.line(alert_label(*kind));
                }
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                r.flush(false);
                r.quotes -= 1;
                r.blank();
            }
            Event::Start(Tag::List(start)) => {
                r.flush(false);
                r.lists.push(*start);
            }
            Event::End(TagEnd::List(_)) => {
                r.flush(false);
                r.lists.pop();
                if r.lists.is_empty() {
                    r.blank();
                }
            }
            Event::Start(Tag::Item) => {
                r.flush(false);
                let marker = match r.lists.last_mut() {
                    Some(Some(number)) => {
                        let m = format!("{number}. ");
                        *number += 1;
                        m
                    }
                    _ => "• ".into(),
                };
                r.marker = Some(marker);
            }
            Event::End(TagEnd::Item) => r.flush(false),
            Event::Start(Tag::Strong) => r.start_style("1"),
            Event::Start(Tag::Emphasis) => r.start_style("3"),
            Event::Start(Tag::Strikethrough) => r.start_style("9"),
            Event::End(TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough) => r.end_style(),
            Event::Code(text) => {
                r.start_style(if options.dark { "38;5;223" } else { "38;5;94" });
                r.buffer.push_str(&safe(text));
                r.end_style();
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                r.links.push(safe(dest_url));
                r.start_style("4");
            }
            Event::End(TagEnd::Link) => {
                r.end_style();
                if let Some(url) = r.links.pop() {
                    r.buffer.push_str(&format!(" ({url})"));
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                r.links.push(safe(dest_url));
                r.buffer.push_str("[image: ");
            }
            Event::End(TagEnd::Image) => {
                if let Some(url) = r.links.pop() {
                    r.buffer.push_str(&format!(" — {url}]"));
                }
            }
            Event::Text(text) => r.buffer.push_str(&safe(text)),
            Event::SoftBreak => r.buffer.push(' '),
            Event::HardBreak => r.buffer.push('\n'),
            Event::Rule => {
                r.flush(true);
                r.line(&"─".repeat(r.options.width.saturating_sub(display_width(&r.prefix()))));
                r.blank();
            }
            Event::TaskListMarker(done) => {
                r.marker = Some(if *done { "☑ ".into() } else { "☐ ".into() });
            }
            Event::Start(Tag::Table(_)) => {
                r.flush(true);
                r.table = Some(vec![vec![]]);
            }
            Event::Start(Tag::TableRow) => {
                if let Some(rows) = &mut r.table {
                    rows.push(vec![]);
                }
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(rows) = &mut r.table {
                    rows.last_mut().unwrap().push(std::mem::take(&mut r.buffer));
                }
            }
            Event::End(TagEnd::Table) => {
                let rows = r.table.take().unwrap();
                r.draw_table(rows);
                r.blank();
            }
            _ => {}
        }
    }
    r.flush(false);
    r.output.trim_end().to_owned() + "\n"
}

fn alert_label(kind: BlockQuoteKind) -> &'static str {
    match kind {
        BlockQuoteKind::Note => "NOTE",
        BlockQuoteKind::Tip => "TIP",
        BlockQuoteKind::Important => "IMPORTANT",
        BlockQuoteKind::Warning => "WARNING",
        BlockQuoteKind::Caution => "CAUTION",
    }
}

struct Renderer {
    options: TerminalOptions,
    output: String,
    buffer: String,
    lists: Vec<Option<u64>>,
    marker: Option<String>,
    quotes: usize,
    styles: Vec<&'static str>,
    heading: Option<u8>,
    in_code: bool,
    code_index: usize,
    links: Vec<String>,
    table: Option<Vec<Vec<String>>>,
}

impl Renderer {
    fn prefix(&self) -> String {
        "│ ".repeat(self.quotes) + &"  ".repeat(self.lists.len().saturating_sub(1))
    }
    fn line(&mut self, text: &str) {
        self.output.push_str(&self.prefix());
        self.output.push_str(text);
        self.output.push('\n');
    }
    fn blank(&mut self) {
        if !self.output.ends_with("\n\n") && !self.output.is_empty() {
            self.output.push('\n');
        }
    }
    fn flush(&mut self, blank: bool) {
        if !self.buffer.is_empty() {
            let text = std::mem::take(&mut self.buffer);
            let base = self.prefix();
            let marker = self.marker.take().unwrap_or_else(|| {
                if self.lists.is_empty() {
                    String::new()
                } else {
                    "  ".into()
                }
            });
            let first = base.clone() + &marker;
            let next = base + &" ".repeat(display_width(&marker));
            let wrapped = textwrap::fill(
                &text,
                textwrap::Options::new(self.options.width)
                    .initial_indent(&first)
                    .subsequent_indent(&next),
            );
            self.output.push_str(&wrapped);
            self.output.push('\n');
        }
        if blank {
            self.blank();
        }
    }
    fn start_style(&mut self, style: &'static str) {
        self.styles.push(style);
        if self.options.ansi {
            self.buffer.push_str(&format!("\x1b[{style}m"));
        }
    }
    fn end_style(&mut self) {
        self.styles.pop();
        if self.options.ansi {
            self.buffer.push_str("\x1b[0m");
            for style in &self.styles {
                self.buffer.push_str(&format!("\x1b[{style}m"));
            }
        }
    }
    fn draw_table(&mut self, rows: Vec<Vec<String>>) {
        let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
        if columns == 0 {
            return;
        }
        let available = self
            .options
            .width
            .saturating_sub(display_width(&self.prefix()));
        if columns * 6 + 1 > available {
            for row in rows.iter().skip(1) {
                for (i, cell) in row.iter().enumerate() {
                    self.buffer = format!(
                        "{}: {cell}",
                        rows[0].get(i).map(String::as_str).unwrap_or("Value")
                    );
                    self.flush(false);
                }
                self.blank();
            }
            return;
        }
        let mut widths = (0..columns)
            .map(|i| {
                rows.iter()
                    .filter_map(|row| row.get(i))
                    .map(|cell| display_width(cell))
                    .max()
                    .unwrap_or(1)
                    .max(3)
            })
            .collect::<Vec<_>>();
        while widths.iter().sum::<usize>() + columns * 3 + 1 > available {
            let i = widths
                .iter()
                .enumerate()
                .max_by_key(|(_, width)| **width)
                .unwrap()
                .0;
            widths[i] -= 1;
        }
        let border = format!(
            "+{}+",
            widths
                .iter()
                .map(|w| "─".repeat(w + 2))
                .collect::<Vec<_>>()
                .join("+")
        );
        self.line(&border);
        for (index, row) in rows.iter().enumerate() {
            let cells = (0..columns)
                .map(|i| textwrap::wrap(row.get(i).map(String::as_str).unwrap_or(""), widths[i]))
                .collect::<Vec<_>>();
            for line in 0..cells.iter().map(Vec::len).max().unwrap_or(1) {
                let content = cells
                    .iter()
                    .enumerate()
                    .map(|(i, cell)| {
                        let value = cell.get(line).map(|v| v.as_ref()).unwrap_or("");
                        format!(
                            " {value}{} ",
                            " ".repeat(widths[i].saturating_sub(display_width(value)))
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("│");
                self.line(&format!("│{content}│"));
            }
            if index == 0 {
                self.line(&border);
            }
        }
        self.line(&border);
    }
}

fn safe(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect::<String>()
        .replace('\t', "    ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_required_blocks_and_inline_styles() {
        let source = include_str!("../../../fixtures/markdown/gfm.md");
        let doc = Document::parse(source);
        let plain = render_terminal(
            &doc,
            TerminalOptions {
                ansi: false,
                width: 60,
                ..Default::default()
            },
        );
        for part in [
            "Marklight",
            "☑",
            "☐",
            "│",
            "•",
            "1.",
            "╭─ rust",
            "https://example.com",
            "Native",
        ] {
            assert!(plain.contains(part), "{part}: {plain}");
        }
        assert!(!plain.contains('\x1b'));
        assert!(!plain.contains("```"));
        assert_eq!(
            plain,
            include_str!("../../../fixtures/markdown/gfm.terminal.txt")
        );
        let styled = render_terminal(&doc, TerminalOptions::default());
        for sequence in ["\x1b[1m", "\x1b[3m", "\x1b[9m"] {
            assert!(styled.contains(sequence));
        }
    }
    #[test]
    fn gfm_alerts_keep_their_kind_in_plain_terminal_output() {
        let doc = Document::parse(include_str!("../../../fixtures/markdown/alerts.md"));
        let plain = render_terminal(
            &doc,
            TerminalOptions {
                ansi: false,
                ..Default::default()
            },
        );
        for label in ["NOTE", "TIP", "IMPORTANT", "WARNING", "CAUTION"] {
            assert!(plain.contains(&format!("│ {label}")), "{plain}");
        }
        assert!(plain.contains("setup guide (basic.md)"));
        assert!(!plain.contains("[!NOTE]"));
        assert!(!plain.contains("<script>"));
    }
    #[test]
    fn unsupported_extensions_remain_visible_in_plain_output() {
        let doc = Document::parse(include_str!("../../../fixtures/markdown/dialect-limits.md"));
        let plain = render_terminal(
            &doc,
            TerminalOptions {
                ansi: false,
                ..Default::default()
            },
        );
        for fragment in [
            "note[^ref]",
            "[^ref]:",
            "title: Not parsed",
            "$x^2$",
            "[[Wiki Link]]",
            "╭─ mermaid",
        ] {
            assert!(plain.contains(fragment), "{fragment}: {plain}");
        }
    }
    #[test]
    fn wraps_unicode_tables_and_long_lines_and_strips_controls() {
        let doc = Document::parse(
            "# 日本語の見出し\n\nThis paragraph has several words that should wrap well at narrow widths.\n\n| Long heading | B |\n|---|---|\n| lots of content with 日本語 | c |\n\n```\nabcdefghijklmnopqrstuvwxyzzzzzzzzzzzzzzz\n```\n\n\x1b]52;c;evil\x07",
        );
        let output = render_terminal(
            &doc,
            TerminalOptions {
                width: 24,
                ansi: false,
                ..Default::default()
            },
        );
        assert!(
            output.lines().all(|line| display_width(line) <= 24),
            "{output}"
        );
        assert!(!output.contains('\x1b'));
        assert!(!output.contains('\x07'));
    }
}
