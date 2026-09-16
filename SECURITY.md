# Security

Treat Markdown files as untrusted. Marklight does not execute document scripts,
code blocks or embedded frames. HTML passes two sanitization steps; the Tauri
window also has a restrictive Content Security Policy. Raw HTML cannot create
copy controls or image grants. Dangerous URL schemes are rejected by core.

There is no telemetry, account, network client or remote-image loading.
Clicking an HTTP(S)/mailto link explicitly delegates it to the system handler;
that action can access the network. Follow only links you trust.

Local image access is limited to referenced raster files within the document
folder tree, checked through canonical paths and served with opaque tokens.
Relative links to other Markdown files open only after an explicit click.
Documents and image requests have size limits. The engine accepts UTF-8 regular
files; it does not read devices or pipes as named documents. Stdin is explicit.

Configuration contains local recent-file paths and reading preferences in the
platform configuration directory. Clear Recent removes that history. Marklight
never changes your Markdown source files. The native application runs with
your user permissions; it is not an OS sandbox against hostile local programs
that can replace files concurrently or tamper with its configuration.

The first macOS build is not Developer ID signed or notarized. Verify checksums
before installation; see the installation guide. No trusted distribution claim
is made for other operating systems until their packages are built and tested.

Please report a vulnerability through the repository's private GitHub security
advisory mechanism when available. Avoid public issues with secrets, private
files or exploitable details. There is no promised response-time SLA.
