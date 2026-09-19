# Security

Treat Markdown files as untrusted. Marklight does not execute document scripts,
code blocks or embedded frames. HTML passes two sanitization steps; the Tauri
window also has a restrictive Content Security Policy. Raw HTML cannot create
copy controls or image grants. Dangerous URL schemes are rejected by core.

There is no telemetry, account, network client or remote-image loading.
Clicking an HTTP(S)/mailto link explicitly delegates it to the system handler;
that action can access the network. Follow only links you trust.

Local image access is limited to referenced raster files within the document
folder tree, checked through canonical paths and served with scoped tokens. The
image request opens through a directory handle captured with the document, so
replacing a referenced file or parent directory with an escaping symlink does
not grant access outside that tree.

Relative links to other Markdown files open only after an explicit click.
They may point outside the current document folder; inspect the destination
before following a link from an untrusted document. HTTP(S) and mailto links
also require a click and open in the system handler.

Documents and image requests have size limits. The engine accepts UTF-8 regular
files; it does not read devices or pipes as named documents. Stdin is explicit.

Configuration contains local recent-file paths and reading preferences in the
platform configuration directory. Clear Recent removes that history. Marklight
never changes your Markdown source files. The native application runs with
your user permissions; it is not an OS sandbox against hostile local programs
that can replace files concurrently or tamper with its configuration.

Run `scripts/audit-deps.sh` before a release and when changing locked
dependencies. It checks the current `Cargo.lock` against the RustSec advisory
database and the desktop `package-lock.json` against npm advisories. The scan
requires network access and a local `cargo-audit` installation; warnings about
unmaintained or unsound transitive packages require review even when the command
exits successfully. Record the scan date, database state and disposition in
`docs/VALIDATION.md`. See the [untrusted-document review](docs/SECURITY-REVIEW.md)
for the current trust boundaries and remaining checks. This is a known-advisory
check, not a code security proof.

The first macOS build is not Developer ID signed or notarized. Verify checksums
before installation; see the installation guide. No trusted distribution claim
is made for other operating systems until their packages are built and tested.

Please report a vulnerability through the repository's private GitHub security
advisory mechanism when available. Avoid public issues with secrets, private
files or exploitable details. There is no promised response-time SLA.
