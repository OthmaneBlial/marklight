# Markdown support

Marklight uses pulldown-cmark 0.13, a CommonMark parser with GFM tables, task
lists, strikethrough and alerts enabled. Bare HTTP(S), www and email autolinks
are recognized in the shared event stream using linkify.

Both interfaces support H1–H6, paragraphs, strong/emphasis/deleted text, inline
code, fenced/indented code, quotes, ordered/unordered/nested lists, horizontal
rules, links, tables and task lists. The desktop additionally displays syntax
highlighting and scoped local raster images. Terminal images are labelled
references. Wide tables wrap cells or use a vertical layout on narrow terminals.

## Deliberate limits

- This is not an editor. It does not execute code or render MDX/JSX.
- Math/LaTeX, Mermaid diagrams, wiki links, footnotes and YAML front matter are
  not interpreted in v0.1. Unsupported syntax stays ordinary document content.
- Raw HTML is omitted in the terminal. The desktop retains basic safe
  formatting; scripts, embedded frames, forms, SVG, styles, event handlers,
  raw-HTML links/images and privileged attributes are removed. This is not a
  general-purpose browser and complex HTML layouts may lose formatting.
- Remote images are blocked. This prevents network requests while opening a
  document. Local PNG/JPEG/GIF/WebP/AVIF/BMP references must resolve inside the
  Markdown file's directory tree. SVG and references outside that tree are
  blocked; an unavailable-image note explains the result.
- Anchors are generated from lowercased heading text, whitespace becomes `-`,
  punctuation is removed, and collisions receive numeric suffixes. Duplicate
  headings are unique; IDs may differ from GitHub for unusual punctuation.
- Local links open `.md`/`.markdown` documents only. Relative parent-directory
  Markdown links are allowed after a user click. Absolute paths, network paths,
  unsafe schemes and links to other local file types are rejected.
- External `http`, `https` and `mailto` links open through the system's default
  browser/mail handler on a user click. They do not navigate the reading WebView.
- Documents must be UTF-8 regular files and are capped at 32 MiB. Local images
  are capped at 16 MiB per request. Recognized code languages are highlighted;
  unknown languages remain plain code. The document still loads as a whole;
  there is no virtualization. Search highlights up to 10,000 matches and shows
  a `+` when that limit is reached.
- `marklight -` supports stdin in the terminal; the GUI requires a saved file.

The fixture directory covers normal, Unicode, nested-list and hostile inputs.
See [security](../SECURITY.md) for the local-document trust boundary.
