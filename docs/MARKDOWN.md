# Markdown support

Marklight uses pulldown-cmark 0.13, a CommonMark parser with GFM tables, task
lists, strikethrough and alerts enabled. Bare HTTP(S), www and email autolinks
are recognized in the shared event stream using linkify.

Both interfaces support H1–H6, paragraphs, strong/emphasis/deleted text, inline
code, fenced/indented code, quotes, ordered/unordered/nested lists, horizontal
rules, links, tables and task lists. The desktop additionally displays syntax
highlighting and scoped local raster images. Terminal images are labelled
references. Wide tables wrap cells or use a vertical layout on narrow terminals.

| Input | Desktop reader | Terminal reader | Checked with |
|---|---|---|---|
| CommonMark headings, lists, code, links | Rendered; generated heading anchors | Rendered; URLs remain visible | `basic.md`, `nested-lists.md`, repository README/guides |
| GFM tables, tasks, deletion, bare autolinks | Rendered, with scrollable wide tables | Rendered, with narrow-table fallback | `gfm.md`, `tables.md`, `links.md` |
| GFM `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]` | Labelled, coloured callouts | Labelled quote blocks | `alerts.md` |
| Local raster image | Displayed only inside the document directory | Alt text and path reference | `images.md` |
| Remote image, executable raw HTML | Blocked or reduced to safe text | Image reference or omitted HTML | `malicious-html.md`, `alerts.md` |
| Footnote syntax, wiki links, inline math | Not interpreted; literal markers remain readable | Not interpreted; literal markers remain readable | `dialect-limits.md` |
| Mermaid fence | Displayed as code, never executed | Displayed as code, never executed | `dialect-limits.md` |
| YAML front matter | Parsed as ordinary Markdown, not metadata | Parsed as ordinary Markdown, not metadata | `dialect-limits.md` |

The fixture table describes the implemented dialect, not GitHub rendering
parity. The first-party `README.md`, `docs/INSTALLATION.md` and
`docs/ARCHITECTURE.md` also have a regression test through both renderers.
Footnotes remain deferred because references and definitions would need
navigation that respects Marklight's generated anchors and link policy. Math,
Mermaid rendering and MDX remain outside this offline, no-execution reader.

## Deliberate limits

- This is not an editor. It does not execute code or render MDX/JSX.
- Math/LaTeX, rendered Mermaid diagrams, wiki links, footnotes and YAML front
  matter are not interpreted. Their syntax remains visible as literal text,
  ordinary Markdown or a fenced code block as shown above.
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
  there is no document virtualization. For documents with more than 400
  headings, the outline shows 100 at a time and offers a heading filter. Search
  highlights up to 10,000 matches and shows a `+` only when more matches exist.
- `marklight -` supports stdin in the terminal; the GUI requires a saved file.

The fixture directory covers normal, Unicode, nested-list and hostile inputs.
See [security](../SECURITY.md) for the local-document trust boundary.
