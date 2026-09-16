use std::{collections::HashSet, sync::LazyLock};

use marklight_core::Document;
use pulldown_cmark::{CowStr, Event, Tag, TagEnd};
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
                let syntax = SYNTAXES.find_syntax_by_token(&block.language).unwrap_or_else(|| SYNTAXES.find_syntax_plain_text());
                let theme = &THEMES.themes[if options.dark { "base16-ocean.dark" } else { "InspiredGitHub" }];
                let mut highlighted = String::new();
                // Bound syntax highlighting cost for very large code blocks.
                if block.code.len() <= 128 * 1024 {
                    let mut highlighter = HighlightLines::new(syntax, theme);
                    for line in LinesWithEndings::from(&block.code) {
                        match highlighter.highlight_line(line, &SYNTAXES)
                            .ok().and_then(|ranges| styled_line_to_highlighted_html(&ranges, IncludeBackground::No).ok()) {
                            Some(html) => highlighted.push_str(&html),
                            None => highlighted.push_str(&escape(line)),
                        }
                    }
                } else { highlighted = escape(&block.code); }
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
}
