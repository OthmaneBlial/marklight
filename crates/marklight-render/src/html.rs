use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use marklight_core::Document;
use pulldown_cmark::{BlockQuoteKind, CowStr, Event, Tag, TagEnd};
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    html::{IncludeBackground, styled_line_to_highlighted_html},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

#[derive(Default)]
pub struct HtmlOptions<'a> {
    /// Map local image references to opaque, scoped URLs. No images by default.
    pub image_url: Option<&'a ImageResolver<'a>>,
    pub dark: bool,
}

type ImageResolver<'a> = dyn Fn(&str) -> Option<String> + 'a;

pub fn render_html(document: &Document, options: &HtmlOptions<'_>) -> String {
    let mut heading = 0;
    let mut code = 0;
    let mut in_code = false;
    let mut code_cache: HashMap<(&str, &str), String> = HashMap::new();
    let mut cached_bytes = 0;
    let raw_tags: HashSet<&str> = [
        "p", "b", "i", "strong", "em", "del", "br", "details", "summary", "kbd", "sub", "sup",
    ]
    .into();
    let mut raw_cleaner = ammonia::Builder::default();
    raw_cleaner
        .tags(raw_tags)
        .generic_attributes(HashSet::new())
        .tag_attributes(Default::default());
    let iter = document.events.iter().filter_map(|event| {
        match event {
            Event::Start(Tag::CodeBlock(_)) => {
                in_code = true;
                let block = &document.code_blocks[code];
                let key = (block.language.as_str(), block.code.as_str());
                let highlighted = if let Some(cached) = code_cache.get(&key) {
                    std::borrow::Cow::Borrowed(cached.as_str())
                } else {
                    let result = highlight_code(&block.code, &block.language, options.dark);
                    if code_cache.len() < 512 && cached_bytes + result.len() <= 4 * 1024 * 1024 {
                        cached_bytes += result.len();
                        code_cache.insert(key, result.clone());
                    }
                    std::borrow::Cow::Owned(result)
                };
                let html = format!("<pre data-code=\"{code}\" data-language=\"{}\"><code>{highlighted}</code></pre>\n", escape(&block.language));
                code += 1;
                Some(Event::Html(html.into()))
            }
            Event::End(TagEnd::CodeBlock) => { in_code = false; None }
            _ if in_code => None,
            Event::Start(Tag::Heading { level, .. }) => {
                let id = &document.headings[heading].id;
                heading += 1;
                Some(Event::Start(Tag::Heading { level: *level, id: Some(CowStr::Borrowed(id)), classes: vec![], attrs: vec![] }))
            }
            Event::Start(Tag::BlockQuote(Some(kind))) => {
                let (class, label) = alert_parts(*kind);
                Some(Event::Html(format!("<blockquote class=\"markdown-alert-{class}\"><p class=\"markdown-alert-title\">{label}</p>").into()))
            }
            Event::End(TagEnd::BlockQuote(Some(_))) => Some(Event::Html("</blockquote>".into())),
            Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
                let url = options.image_url.and_then(|resolve| resolve(dest_url)).unwrap_or_default();
                Some(Event::Start(Tag::Image { link_type: *link_type, dest_url: url.into(), title: CowStr::Borrowed(title), id: CowStr::Borrowed(id) }))
            }
            Event::Html(html) | Event::InlineHtml(html) => Some(Event::Html(raw_cleaner.clean(html).to_string().into())),
            // Borrow common large text events rather than cloning their allocations.
            Event::Text(text) => Some(Event::Text(CowStr::Borrowed(text))),
            Event::Code(text) => Some(Event::Code(CowStr::Borrowed(text))),
            _ => Some(event.clone()),
        }
    });
    let mut html = String::with_capacity(document.metadata.bytes);
    pulldown_cmark::html::push_html(&mut html, iter);
    let mut cleaner = ammonia::Builder::default();
    cleaner
        .add_generic_attributes(["id", "class"])
        .add_tag_attributes("span", ["style"])
        .add_tag_attributes("pre", ["data-code", "data-language"])
        .add_tags(["input", "details", "summary"])
        .add_tag_attributes("input", ["type", "checked", "disabled"])
        .url_schemes(["http", "https", "mailto", "marklight-image"].into())
        .link_rel(Some("noreferrer noopener"));
    cleaner.clean(&html).to_string()
}

fn alert_parts(kind: BlockQuoteKind) -> (&'static str, &'static str) {
    match kind {
        BlockQuoteKind::Note => ("note", "Note"),
        BlockQuoteKind::Tip => ("tip", "Tip"),
        BlockQuoteKind::Important => ("important", "Important"),
        BlockQuoteKind::Warning => ("warning", "Warning"),
        BlockQuoteKind::Caution => ("caution", "Caution"),
    }
}

fn highlight_code(code: &str, language: &str, dark: bool) -> String {
    let syntax = SYNTAXES
        .find_syntax_by_token(language)
        .unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());
    if syntax.name == "Plain Text" {
        return escape(code);
    }
    let theme = &THEMES.themes[if dark {
        "base16-ocean.dark"
    } else {
        "InspiredGitHub"
    }];
    let mut result = String::new();
    let mut highlighter = HighlightLines::new(syntax, theme);
    for line in LinesWithEndings::from(code) {
        match highlighter
            .highlight_line(line, &SYNTAXES)
            .ok()
            .and_then(|ranges| styled_line_to_highlighted_html(&ranges, IncludeBackground::No).ok())
        {
            Some(html) => result.push_str(&html),
            None => result.push_str(&escape(line)),
        }
    }
    result
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TerminalOptions, render_terminal};
    #[test]
    fn repository_readme_and_guides_render_in_both_interfaces() {
        for source in [
            include_str!("../../../README.md"),
            include_str!("../../../docs/INSTALLATION.md"),
            include_str!("../../../docs/ARCHITECTURE.md"),
        ] {
            let doc = Document::parse(source);
            assert!(!doc.headings.is_empty());
            let html = render_html(&doc, &HtmlOptions::default());
            let plain = render_terminal(
                &doc,
                TerminalOptions {
                    ansi: false,
                    ..Default::default()
                },
            );
            assert!(html.contains("<h1"), "{}", doc.metadata.title);
            assert!(
                plain.contains(&doc.metadata.title),
                "{}",
                doc.metadata.title
            );
        }
    }
    #[test]
    fn large_code_is_highlighted_and_repeated_blocks_keep_distinct_copy_ids() {
        let source = "let value = 42;\n".repeat(9000);
        assert!(source.len() > 128 * 1024);
        let doc = Document::parse(&format!("```rust\n{source}```\n\n```rust\n{source}```\n"));
        let html = render_html(&doc, &HtmlOptions::default());
        assert!(html.contains("data-code=\"0\""));
        assert!(html.contains("data-code=\"1\""));
        assert!(html.matches("<span style=").count() > 9000);
        assert_eq!(doc.code_blocks[0].code, source);
        assert_eq!(doc.code_blocks[1].code, source);
    }
    #[test]
    fn sanitizes_hostile_html_urls_and_images() {
        let doc = Document::parse(include_str!("../../../fixtures/markdown/malicious-html.md"));
        let html = render_html(&doc, &HtmlOptions::default());
        for forbidden in [
            "<script",
            "<iframe",
            "onerror=",
            "onclick=",
            "javascript:",
            "file:///",
            "data:text",
            "<svg",
            "http://tracker",
        ] {
            assert!(!html.contains(forbidden), "{forbidden}: {html}");
        }
        assert!(html.contains("Safe text"));
    }
    #[test]
    fn alerts_have_visible_safe_labels_and_keep_local_links() {
        let doc = Document::parse(include_str!("../../../fixtures/markdown/alerts.md"));
        let html = render_html(&doc, &HtmlOptions::default());
        for (kind, label) in [
            ("note", "Note"),
            ("tip", "Tip"),
            ("important", "Important"),
            ("warning", "Warning"),
            ("caution", "Caution"),
        ] {
            assert!(
                html.contains(&format!("class=\"markdown-alert-{kind}\"")),
                "{html}"
            );
            assert!(html.contains(&format!(">{label}</p>")), "{html}");
        }
        assert!(html.contains("href=\"basic.md\""));
        assert!(!html.contains("<script"));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("onclick="));
    }
    #[test]
    fn unsupported_extensions_stay_inert_and_readable() {
        let doc = Document::parse(include_str!("../../../fixtures/markdown/dialect-limits.md"));
        let html = render_html(&doc, &HtmlOptions::default());
        assert!(html.contains("note[^ref]"), "{html}");
        assert!(html.contains("[[Wiki Link]]"), "{html}");
        assert!(html.contains("data-language=\"mermaid\""), "{html}");
        assert!(!html.contains("<svg"));
        assert!(!html.contains("<script"));
    }
    #[test]
    fn renders_unique_anchors_scoped_images_and_highlighted_code() {
        let doc = Document::parse(
            "# Hello\n# Hello\n\n![Photo](./a.png)\n\n```rust\nfn main() {}\n```\n",
        );
        let resolver =
            |url: &str| (url == "./a.png").then(|| "marklight-image://localhost/token".into());
        let html = render_html(
            &doc,
            &HtmlOptions {
                image_url: Some(&resolver),
                dark: false,
            },
        );
        assert!(html.contains("id=\"hello-1\""));
        assert!(html.contains("marklight-image://localhost/token"));
        assert!(html.contains("data-code=\"0\""));
        assert!(html.contains("data-language=\"rust\""));
        assert!(html.contains("<span style="));
    }
    #[test]
    fn raw_html_cannot_forge_privileged_attributes_or_code_elements() {
        let doc = Document::parse(
            "<pre data-code=\"0\"><code>forged</code></pre>\n\n<p id=\"unsafe\" style=\"color:red\" onclick=\"x()\">Safe text</p>",
        );
        let html = render_html(&doc, &HtmlOptions::default());
        assert!(!html.contains("data-code"));
        assert!(!html.contains("id=\"unsafe\""));
        assert!(!html.contains("style="));
    }

    #[test]
    fn bounded_hostile_markup_mutations_do_not_gain_active_html() {
        let payloads = [
            "<script>alert(1)</script>",
            "<img src=\"https://tracker.invalid/pixel\" onerror=\"alert(1)\">",
            "<svg onload=\"alert(1)\"><script>alert(1)</script></svg>",
            "[click](javascript:alert(1))",
            "<pre data-code=\"0\"><code>forged</code></pre>",
        ];
        let wrappers = ["{payload}", "> {payload}", "- {payload}", "# {payload}"];
        for payload in payloads {
            for wrapper in wrappers {
                let source = wrapper.replace("{payload}", payload);
                let html = render_html(&Document::parse(&source), &HtmlOptions::default());
                for forbidden in [
                    "<script",
                    "<svg",
                    "onerror=",
                    "onload=",
                    "src=\"https://",
                    "javascript:",
                    "data-code=\"0\"",
                ] {
                    assert!(!html.contains(forbidden), "{source:?}: {html}");
                }
            }
        }
    }
}
