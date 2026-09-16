# Implementation and acceptance plan

The source of scope is the supplied Marklight mission, sections 1–41. This is
a reader: terminal and desktop share the same Rust Markdown event stream.

1. Workspace and public repository: stable Rust, main branch, explicit staged
   paths, meaningful validated commits pushed throughout. No GitHub Actions.
2. Core: GFM parsing, headings and unique anchors, links, code originals,
   metadata, directory defaults, UTF-8 errors, bounded loading, platform TOML
   preferences and recent paths, notify watcher abstraction. Unit tests and
   the named Markdown fixtures prove each contract.
3. Rendering: all required terminal blocks and inline styles, Unicode wrapping,
   tables, nested/task lists, safe terminal controls; HTML with allowlist
   sanitization, safe schemes and syntax highlighting. Golden and adversarial
   tests. No parsing duplication in the interfaces.
4. CLI: paths, directories, stdin, width, plain output, pager/search/q,
   theme, actionable errors and exit codes, desktop launch. Integration tests
   invoke the actual executable and validate output and failure behavior.
5. Desktop: Tauri 2, tiny TypeScript UI; native open menu/dialog/drop/OS opening,
   single instance, local relative links and scoped images, selectable reading
   text, exact code copying, TOC, search, themes, zen, font shortcuts, recent
   paths, live reload with reading context preserved. Rust command tests,
   frontend tests, browser interaction checks, and actual native runtime checks.
6. Measure: repeatable 10 KB, 100 KB, 1 MB, 5 MB and 10 MB parsing/rendering
   measurements; executable startup, desktop startup, idle CPU and memory.
   Publish measured values with hardware and scope, never infer performance.
7. Documentation and distribution: README, changelog, contribution and security
   guidance, dialect limitations, installation, native file associations,
   verified native packages where the host supports them. Other operating
   systems stay explicitly unverified until tested on those systems.
8. Publication: public source repository with metadata and verified main HEAD;
   repository-grounded website and docs; initial GitHub release only after
   local gates and package checks. Registry publication requires separate
   credentials and ownership evidence; document source installation meanwhile.

Required local milestone gates are `cargo fmt --check`, strict workspace Clippy,
workspace tests, and applicable frontend checks. No Docker, backend service,
database, cloud dependency, React, or Apple tooling installation.

## Evidence

Record current acceptance evidence in `docs/VALIDATION.md`. Build success is not
proof of a working native UI, another platform, or a published registry package.
