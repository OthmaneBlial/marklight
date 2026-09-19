# Release contract for the next Marklight version

Drafted 2026-09-19 from the `v0.1.1` baseline. This is a gate for a future
release, **not** evidence that the gate has passed. Refer to
[validation](VALIDATION.md), [performance](PERFORMANCE.md) and
[the roadmap](../ROADMAP.md) for the current state.

## Supported-platform decision

| Platform | CLI evidence today | Desktop/package evidence today | Next-release rule |
|---|---|---|---|
| macOS arm64 | Local tests and npm smoke pass; public 0.1.1 metadata exists | Historical native 0.1.1 checks and local app/DMG checks; ad hoc signature | Download and install the **new** public asset on a clean profile, validate file opening and Gatekeeper outcome, then label limitations accurately. |
| macOS x86_64 | Source install is described, native run not verified | No verified installer | Announce only after an x86_64 native machine or runner has installed and exercised the package. |
| Windows x64 | Source build not verified here | No verified installer | Announce only after a native install, file association, open/read/copy/reload and uninstall test. |
| Linux x64 | Source build not verified here | No verified package | Announce only after native package and desktop-environment tests; name the distribution tested. |

The scope of a release may be macOS arm64 alone if other hosts are unavailable.
Do not generate a download labelled supported merely because it compiled.

## Quality gate

- [ ] Record target commit, tag, source tree status, compiler/Node/Tauri versions and OS.
- [ ] Run `./scripts/check.sh`, all-feature Clippy, pager smoke, npm package smoke,
      site check, adversarial fixtures and the new navigation/performance tests.
- [ ] Run the performance protocol in `docs/PERFORMANCE.md`, including at least
      three fresh native launches per size, one varied guide, interactive
      search/outline timing and complete process-tree memory scope.
- [ ] On every announced OS, install **the actual packaged asset** in a clean
      environment and test launch, `.md` open, links, search, copy, reload,
      keyboard controls and exit. Label assistive-technology checks separately.
- [ ] Decide with the maintainer whether GitHub Actions are allowed. Current
      `scripts/check.sh`, `docs/PLAN.md` and `CONTRIBUTING.md` intentionally
      prohibit workflows. If they remain disabled, attach dated manual gate
      output for the release; never display a CI badge without a real run.

## Packaging and trust gate

- [ ] Verify the same version in Cargo workspace, Tauri, desktop npm and CLI
      npm manifests, the built executable, changelog and release notes.
- [ ] Build assets from the chosen commit; record their SHA-256 in
      `SHA256SUMS`; inspect archive contents and test extraction/installation.
- [ ] If Developer ID signing and notarization are available, use the owner's
      protected credentials, then verify the downloaded DMG with Gatekeeper.
      Otherwise keep the unsigned/ad hoc limitation prominent. Never put
      credentials in Git, artifacts or shared logs.
- [ ] Confirm registry ownership and availability before publishing crates.io
      or changing npm platform claims. Registry access belongs to the owner.
- [ ] Check release assets, checksums, README/site/npm links and installation
      commands **after** publication from a fresh download. Upload completion
      alone is not a passed release.

## Ownership and external gates

The maintainer decides CI policy, grants any signing identity and registry
credentials, and authorizes GitHub Release, npm/crates.io and website
publication. Source pushes to `main` are authorized in the current work
request; a new public release or registry publication is not. Native Windows,
Linux and Intel Mac validation needs access to those systems. Log each unmet
external gate as blocked rather than marking its checklist item complete.
