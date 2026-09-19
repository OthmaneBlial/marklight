use std::collections::{HashMap, HashSet};

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Link {
    pub destination: String,
    pub image: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metadata {
    pub bytes: usize,
    pub words: usize,
    pub reading_minutes: usize,
    pub title: String,
}

/// One shared event stream; renderers never reparse Markdown.
#[derive(Debug)]
pub struct Document {
    pub events: Vec<Event<'static>>,
    pub headings: Vec<Heading>,
    pub links: Vec<Link>,
    pub code_blocks: Vec<CodeBlock>,
    pub metadata: Metadata,
}

impl Document {
    pub fn parse(source: &str) -> Self {
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_GFM;
        let mut events = Vec::new();
        let mut headings = Vec::new();
        let mut links = Vec::new();
        let mut code_blocks = Vec::new();
        let mut heading: Option<(u8, String)> = None;
        let mut code: Option<CodeBlock> = None;
        let mut code_last_end: Option<usize> = None;
        let mut occupied = HashSet::new();
        let mut next_suffix: HashMap<String, usize> = HashMap::new();
        let mut in_link = 0;
        let mut words = 0;
        let mut finder = linkify::LinkFinder::new();
        finder.kinds(&[linkify::LinkKind::Url, linkify::LinkKind::Email]);

        for (event, range) in Parser::new_ext(source, options).into_offset_iter() {
            match &event {
                Event::Start(Tag::Heading { level, .. }) => {
                    heading = Some((*level as u8, String::new()));
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some((level, text)) = heading.take() {
                        let base = slug(&text);
                        let suffix = next_suffix.entry(base.clone()).or_default();
                        let mut id = if *suffix == 0 {
                            base.clone()
                        } else {
                            format!("{base}-{suffix}")
                        };
                        while !occupied.insert(id.clone()) {
                            *suffix += 1;
                            id = format!("{base}-{suffix}");
                        }
                        *suffix += 1;
                        headings.push(Heading { level, text, id });
                    }
                }
                Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => {
                    in_link += 1;
                    links.push(Link {
                        destination: dest_url.to_string(),
                        image: matches!(event, Event::Start(Tag::Image { .. })),
                    });
                }
                Event::End(TagEnd::Link | TagEnd::Image) => in_link -= 1,
                Event::Start(Tag::CodeBlock(kind)) => {
                    code_last_end = None;
                    code = Some(CodeBlock {
                        language: match kind {
                            CodeBlockKind::Fenced(info) => {
                                info.split_whitespace().next().unwrap_or("").to_owned()
                            }
                            CodeBlockKind::Indented => String::new(),
                        },
                        code: String::new(),
                    });
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(block) = code.take() {
                        code_blocks.push(block);
                    }
                }
                Event::Text(text) | Event::Code(text) => {
                    words += text.split_whitespace().count();
                    if let Some((_, title)) = &mut heading {
                        title.push_str(text);
                    }
                    if let Some(block) = &mut code {
                        // Preserve source bytes, including CRLF and trailing newline.
                        if let Some(previous) = code_last_end {
                            let gap = &source[previous..range.start];
                            if gap.chars().all(|c| c == '\r' || c == '\n') {
                                block.code.push_str(gap);
                            }
                        }
                        block.code.push_str(&source[range.clone()]);
                        code_last_end = Some(range.end);
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if let Some((_, title)) = &mut heading {
                        title.push(' ');
                    }
                }
                _ => {}
            }
            // GFM bare autolinks, only in normal text (never inside code/links).
            if let Event::Text(text) = &event
                && code.is_none()
                && in_link == 0
            {
                let mut end = 0;
                for link in finder.links(text) {
                    if link.start() > end {
                        events.push(Event::Text(text[end..link.start()].to_owned().into()));
                    }
                    let destination = match link.kind() {
                        linkify::LinkKind::Email => format!("mailto:{}", link.as_str()),
                        _ if link.as_str().starts_with("www.") => {
                            format!("https://{}", link.as_str())
                        }
                        _ => link.as_str().to_owned(),
                    };
                    links.push(Link {
                        destination: destination.clone(),
                        image: false,
                    });
                    events.push(Event::Start(Tag::Link {
                        link_type: pulldown_cmark::LinkType::Autolink,
                        dest_url: destination.into(),
                        title: "".into(),
                        id: "".into(),
                    }));
                    events.push(Event::Text(link.as_str().to_owned().into()));
                    events.push(Event::End(TagEnd::Link));
                    end = link.end();
                }
                if end > 0 {
                    if end < text.len() {
                        events.push(Event::Text(text[end..].to_owned().into()));
                    }
                    continue;
                }
            }
            events.push(event.into_static());
        }
        let title = headings
            .first()
            .map(|h| h.text.clone())
            .unwrap_or_else(|| "Untitled document".into());
        Self {
            events,
            headings,
            links,
            code_blocks,
            metadata: Metadata {
                bytes: source.len(),
                words,
                reading_minutes: words.div_ceil(220).max(1),
                title,
            },
        }
    }
}

fn slug(text: &str) -> String {
    let mut result = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            result.push(c);
        } else if c.is_whitespace() {
            result.push('-');
        }
    }
    if result.is_empty() {
        "section".into()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_have_inline_text_unicode_and_collision_free_ids() {
        let doc = Document::parse(
            "# Hello, **world**!\n## Hello, world!\n### hello-world-1\n#### 日本語\n##### Five\n###### Six",
        );
        assert_eq!(
            doc.headings
                .iter()
                .map(|h| h.id.as_str())
                .collect::<Vec<_>>(),
            [
                "hello-world",
                "hello-world-1",
                "hello-world-1-1",
                "日本語",
                "five",
                "six"
            ]
        );
        assert_eq!(
            doc.headings.iter().map(|h| h.level).collect::<Vec<_>>(),
            [1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn extracts_links_and_gfm_without_linkifying_code() {
        let doc = Document::parse(
            "[Guide](guide.md) ![Photo](assets/a.png) <https://example.org> https://example.com `https://code.test`\n\n- [x] Done\n\n~~gone~~\n\n| A | B |\n|---|---|\n| 1 | 2 |",
        );
        assert_eq!(doc.links.len(), 4);
        assert!(doc.links[1].image);
        assert!(doc.events.contains(&Event::TaskListMarker(true)));
        assert!(doc.events.contains(&Event::Start(Tag::Strikethrough)));
        assert!(
            doc.events
                .iter()
                .any(|e| matches!(e, Event::Start(Tag::Table(_))))
        );
    }

    #[test]
    fn classifies_gfm_alerts_without_losing_local_links() {
        let doc = Document::parse(include_str!("../../../fixtures/markdown/alerts.md"));
        let kinds: Vec<_> = doc
            .events
            .iter()
            .filter_map(|event| match event {
                Event::Start(Tag::BlockQuote(kind)) => *kind,
                _ => None,
            })
            .collect();
        assert_eq!(
            kinds,
            [
                pulldown_cmark::BlockQuoteKind::Note,
                pulldown_cmark::BlockQuoteKind::Tip,
                pulldown_cmark::BlockQuoteKind::Important,
                pulldown_cmark::BlockQuoteKind::Warning,
                pulldown_cmark::BlockQuoteKind::Caution,
            ]
        );
        assert!(doc.links.iter().any(|link| link.destination == "basic.md"));
    }

    #[test]
    fn repeated_heading_ids_scale_and_remain_unique() {
        let doc = Document::parse(&"# Repeated\n".repeat(10000));
        assert_eq!(doc.headings[9999].id, "repeated-9999");
        assert_eq!(
            doc.headings
                .iter()
                .map(|h| &h.id)
                .collect::<HashSet<_>>()
                .len(),
            10000
        );
    }

    #[test]
    fn preserves_exact_fenced_code() {
        let doc =
            Document::parse("```rust\r\nfn main() {\r\n    println!(\"hi\");\r\n}\r\n```\r\n");
        assert_eq!(doc.code_blocks[0].language, "rust");
        assert_eq!(
            doc.code_blocks[0].code,
            "fn main() {\r\n    println!(\"hi\");\r\n}\r\n"
        );
    }
}
