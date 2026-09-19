# Contributing

Use Rust stable and Node 22.12+ (or a supported newer Node). Tauri's native
prerequisites are needed for desktop workspace tests; see its official guide:
https://v2.tauri.app/start/prerequisites/.

```sh
npm --prefix apps/desktop ci
npm --prefix apps/desktop run build
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo run -p marklight-render --example frontend_fixtures
npm --prefix apps/desktop test
node scripts/check-site.mjs
```

Install a Playwright Chromium locally with `npx playwright install chromium`
from `apps/desktop`, or set `MARKLIGHT_CHROMIUM` to an existing compatible
Chromium executable. No browser download is required for CLI-only development.
`./scripts/check.sh` runs the complete local gate. There are intentionally no
GitHub Actions and no automatic CI runs. Do not add workflows without approval.
The gate prints its source revision, host and tool versions. Record its full
output on the exact release commit; it does not build or validate a public
download. Packaging, dependency review and native installation remain separate
release checks in `docs/RELEASE-CHECKLIST.md`.

For CLI-only changes, `cargo test` and `cargo clippy --all-targets -- -D warnings`
use the default members and do not require desktop prerequisites. For desktop
development use `npm --prefix apps/desktop run tauri -- dev`.

Keep parsing and path policy in core, renderers independent of Tauri, and the
frontend small. Preserve ordinary text selection, offline loading and keyboard
access. Add meaningful tests for behavior/security changes. Golden changes
should be reviewed as visible output, not blindly regenerated.

Submit a focused pull request describing the problem, user-visible behavior
and local checks. Attach screenshots for UI changes. Do not include local paths,
personal documents, credentials or generated build artifacts. Contributions
are made under the project's MIT license.
