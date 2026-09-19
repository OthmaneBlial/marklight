# Untrusted document boundary — development review

Reviewed on 2026-09-19 during phase 2.1 (not a release audit).
Marklight runs with the user's file permissions. The review concerns what an
opened Markdown document can cause without an additional user action.

| Entry | Implemented boundary | Evidence and remaining limit |
|---|---|---|
| Markdown and raw HTML | `crates/marklight-render/src/html.rs` filters raw HTML before converting parser events to HTML, then sanitizes the complete result. Generated code HTML uses escaped language/code content; GFM alert classes and labels come from a fixed enum. | `malicious-html.md`, `alerts.md` and renderer tests reject scripts, handlers, frames, SVG, privileged URLs and raw remote images. Browser tests check that hostile documents issue no remote request. Sanitization remains a dependency that must be reviewed when upgraded. |
| WebView | `apps/desktop/src-tauri/tauri.conf.json` limits scripts to packaged sources, blocks frames, objects and form submission, and limits image and IPC origins. `main.ts` inserts only the Rust-rendered, sanitized payload. | CSP is defense in depth, not a replacement for the Rust sanitizer. The current local macOS build displayed the adversarial alert fixture; other operating systems remain unverified. |
| Links | `Reader::navigate` accepts only destinations parsed from the current document; `classify_link` rejects control characters, unsafe schemes, absolute file paths and non-Markdown local targets. HTTP(S)/mailto open only through the system handler after a click. | Core tests cover percent encoding, fragments and schemes. A relative Markdown link may traverse to a parent folder after a click; this is deliberate and documented in `SECURITY.md`. The destination should be shown clearly in the reader before use. |
| Images | Only parsed image references can receive a current-document token. `resolve_image` limits initial grants to local raster paths under the document folder. `Reader::image` opens the saved path through a [`cap-std` directory handle](https://docs.rs/cap-std/4.0.3/cap_std/fs/struct.Dir.html) captured on open/reload, checks regular-file status and reads at most 16 MiB + 1 byte before rejecting oversize content. | Rust tests replace the leaf, image parent and document folder with symlinks after token creation; no outside bytes are returned. The local macOS WebKit build requested the local raster through the native image protocol and showed the remote image as unavailable. The native symlink attack and other operating systems remain unverified. An attacker already running as the same user can modify in-scope file contents. |
| Resource bounds | `read_source` caps documents at 32 MiB; image reads cap at 16 MiB; search displays at most 10,000 marked matches. | Unit/browser tests cover these boundaries. 10 MiB Markdown can still produce a large DOM and high process memory; see `PERFORMANCE.md`. |

Run `scripts/audit-deps.sh` before each release and lockfile update. The
[RustSec advisory database](https://rustsec.org/) and npm audits catch known
reports, not undisclosed defects. On 2026-09-19 the scan found zero known
vulnerabilities and nine informational RustSec warnings. Their dependency
paths and platform relevance are recorded in `VALIDATION.md`; the Linux GTK
warning needs a fresh decision before any Linux package is claimed.

The bounded mutation tests in `paths.rs` cover 32 raw/percent-encoded
traversal combinations with mixed-case escapes. The HTML renderer test feeds
five hostile payloads through four Markdown contexts. These are deterministic
regressions against known classes of mistakes, not a coverage proof or a
replacement for a sanitizer audit.

Remaining phase 2.1 work: review URL and file-opening behavior on every
announced OS, and rerun the adversarial corpus and dependency audit on the
final release commit and binaries. These are release gates, not inferred from
this review.
