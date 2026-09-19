# Performance — measured, not inferred

## Recheck of 0.1.1 on 2026-09-19

The source changes since tag `v0.1.1` were documentation only when this
baseline was captured. The tested CLI and desktop binaries report 0.1.1;
their SHA-256 hashes and the raw samples are in
[measurements-0.1.1-baseline.json](measurements-0.1.1-baseline.json). The
release-mode kernel example was rerun from the checkout. The desktop executable
was the existing local 0.1.1 bundle, **not** a newly downloaded public asset.
The full local `scripts/check.sh` gate, pager smoke, npm tarball smoke and site
check passed. This does not establish performance or installation on another OS.

| Document | Parse | Terminal render | HTML render | Native entry → DOM/layout ready |
|---:|---:|---:|---:|---:|
| 10 KB | 0.23 ms | 1.00 ms | 2.35 ms | 565 ms |
| 100 KB | 2.05 ms | 8.59 ms | 20.17 ms | 623 ms |
| 1 MB | 18.26 ms | 84.27 ms | 202.77 ms | 1,204 ms |
| 5 MB | 110.21 ms | 589.70 ms | 1,429.68 ms | 10,367 ms |
| 10 MB | 253.30 ms | 821.94 ms | 1,724.57 ms | 9,719 ms |

The kernel figures are **one sample per size** from the repeated GFM corpus;
they exclude I/O, IPC and WebView layout. The native figures are also one
fresh launch per size. The 5 MB value exceeding 10 MB illustrates run-to-run
noise and is not a scaling law. Three additional 357-byte launches measured
1,010, 556 and 559 ms from native entry to DOM/layout ready; the first run
was colder. This still is not first visible paint. Twenty CLI `--help` runs
had a 6.33 ms median; twenty small-file reads had a 6.06 ms median. Five idle
native-process RSS samples were 110–111 MiB; WebKit helpers were excluded.
The published 0.1.0 observations below remain a separate historical baseline.

The next release has **provisional M2/macOS arm64 goals**, to be checked on
three fresh runs per workload and at least one varied real guide: 1 MB ready
within 1.5 s, 5 MB within 5 s, 10 MB within 8 s, warm small-document ready
within 500 ms, and visible response to search/outline input within 200 ms on
1 MB and 500 ms on 10 MB. These are user-experience targets, not achieved
results or guarantees on other hardware. Interaction and complete process-tree
memory baselines still need instrumentation before those goals can be judged.

## Repeated native development profile on 2026-09-19

On clean source commit `2e97569`, a local release-mode macOS arm64 app and CLI
were identified by SHA-256 in [the raw repeated samples](measurements-phase1-repeated.json).
The benchmark opened each generated size in **three fresh native processes**,
without compiling or compressing concurrently. A real, varied guide from this
repository (`docs/INSTALLATION.md`) also opened in 520 ms on one run. These
measurements are from the local development build, not a downloaded package.

| Markdown bytes | Native entry → DOM/layout ready, three runs | Median | Provisional goal |
|---:|---:|---:|---:|
| 10 KB | 499 / 502 / 493 ms | 499 ms | — |
| 100 KB | 569 / 557 / 559 ms | 559 ms | — |
| 1 MB | 1,180 / 1,196 / 1,164 ms | 1,180 ms | ≤1,500 ms |
| 5 MB | 11,131 / 10,286 / 10,114 ms | 10,286 ms | ≤5,000 ms |
| 10 MB | 22,598 / 22,910 / 18,418 ms | 22,598 ms | ≤8,000 ms |

The 1 MB goal passed on this corpus; **5 and 10 MB failed**. The first of three
357-byte launches took 1,090 ms and the next two 509/478 ms, so the cold
startup and warm 500 ms goals are not established. For 10 MB, the frontend
reported a median of 19,706 ms inside the overall 22,598 ms readiness time.
Its broad `outline_ms` bucket accounted for a median 12,682 ms, while initial
IPC took 2,326 ms, template construction 2,367 ms and progress calculation
1,546 ms. The broad outline bucket includes showing the document, focus,
heading indexing and outline painting.

A later, **uncommitted diagnostic build** split that bucket on one 10 MB run:
native readiness was 21,955 ms, with 11,757 ms in `viewport.focus()` after DOM
attachment, 65 ms indexing the outline and 20 ms painting its first page.
Moving focus before attachment gave three readiness samples of 17,676 / 23,760
/ 22,337 ms (median 22,337 ms). Focus then took approximately 0 ms, but the
subsequent `scrollHeight`/progress step took 11,186 / 14,925 / 13,580 ms.
This moved the forced layout cost without meeting the 8-second goal, so the
focus placement was restored. A second uncommitted experiment halved document
chunk size to 100 blocks. Its first 10 MB launch took 21,999 ms; the second
did not report readiness within 90 seconds. The chunk-size change was also
restored. These experiments do not establish a browser defect or improvement;
they rule out those two small changes as release fixes on this host.

This intentionally dense corpus repeats headings, tables and code and is not
representative of every Markdown guide. The benchmark measures native entry to
DOM/layout readiness, **not first visible paint**, interaction latency, smooth
scrolling or total app-plus-WebKit memory. The idle RSS sample covers only the
native process. Those missing measurements remain release gates for phase 1.1.

## Deferred initial layout profile on 2026-09-19

Commit `5e646fa` defers the initial reader focus and progress calculation until
after the first interactive render. This removes the forced layout from the
startup gate while keeping the viewport focused and the progress indicator
updated on the next task. A release-mode macOS arm64 app with executable SHA-256
`35ad8076da411bc1c3be336f17dbb9c92bdf6599f8d04aa1ce2a7ac915aa332e` was
benchmarked from a clean source tree. Raw samples are
[1 MB](measurements-phase1-deferred-1m-clean.json),
[5 MB](measurements-phase1-deferred-5m-clean.json) and
[10 MB](measurements-phase1-deferred-clean.json).

| Markdown bytes | Native entry → interactive ready, three fresh launches | Median | Provisional goal |
|---:|---:|---:|---:|
| 1,000,000 | 1,214 / 1,173 / 1,165 ms | 1,173 ms | ≤1,500 ms |
| 5,000,000 | 3,926 / 3,957 / 3,935 ms | 3,935 ms | ≤5,000 ms |
| 10,000,000 | 7,964 / 8,068 / 7,998 ms | 7,998 ms | ≤8,000 ms |

The 10 MB median now meets the provisional warm interactive target by about
2 ms, but the cold first launch in an earlier three-run sequence took 70,313 ms
before the same frontend became ready. The cold-start target therefore remains
unmet. The frontend timing for the warm 10 MB runs was about 1,678–1,696 ms;
focus and progress were intentionally outside that gate and completed after the
initial task. The native WebKit accessibility checks still showed a working
search result (`0 matches`) and an outline window of `1–100 of 40,817`.

The benchmark now records RSS for the native process tree as well as the native
process. On the 1 MB clean run the values were 184,000–184,176 KiB and matched
the native-only sample; WebKit helpers were not descendants of that PID on this
host, so this is **not** a complete app-plus-WebKit memory measurement. First
visible paint, controlled search/click/scroll latency, and cold-start mitigation
remain release gates for phase 1.1.

## Phase 1 development profile on 2026-09-19

The current development branch uses visible-chunk heading lookup and displays
at most 100 outline links at a time, with filtering and pagination to reach
every heading. The benchmark now rejects a native “ready” report if the
frontend did not finish rendering. One run for each size and code state is in
[the raw experimental samples](measurements-phase1-progress.json). These are
**development experiments**, not release results; neither the source states
nor the machine load were held constant across all runs.

| Source state | 1 MB ready | 5 MB ready | 10 MB ready |
|---|---:|---:|---:|
| Visible-chunk lookup, full outline | 1,260 ms | 4,327 ms | 9,132 ms |
| 100-link outline window | 1,231 ms | 4,154 ms | 8,482 ms |
| Windowed outline, split timing profile | 1,978 ms | 7,053 ms | 16,168 ms |
| Conditional scroll reset, first run | 2,544 ms | 8,666 ms | 14,331 ms |
| Conditional scroll reset, repeat | 2,406 ms | 8,106 ms | 17,269 ms |

The split profile attributes most of the late cost to forced WebView layout:
assigning `scrollTop = 0` cost 8,012 ms for 10 MB in one run. Skipping that
unnecessary assignment on initial open moved the cost to the subsequent
`scrollHeight`/active-heading progress calculation (6,744 and 7,981 ms in two
runs). This is a causal observation about **where the synchronous work occurs**,
not evidence that total startup is faster. The 10 MB corpus contains about
40,817 headings. System load was high during later runs, so the 8-second goal
is **not verified**. First visible paint, varied real guides, search/scroll
latency and full app-plus-WebKit memory still need measurement.

Measured 2026-09-16 on Apple M2 / arm64, macOS 26.6, Rust stable 1.95, release
build with thin LTO. Raw samples and host metadata are in
[measurements.json](measurements.json). These are development-host measurements,
not a cross-platform latency guarantee or a memory ceiling. These samples were
recorded for the initial 0.1.0 development build. Version 0.1.1 changes desktop
layout and recent-file switching; the table is not a benchmark of that patch.

## Shared Rust kernels

| Markdown bytes | Parse | Plain terminal render | Sanitized, highlighted HTML |
|---:|---:|---:|---:|
| 10,000 | 0.19 ms | 0.94 ms | 2.09 ms |
| 100,000 | 1.77 ms | 7.25 ms | 17.22 ms |
| 1,000,000 | 15.31 ms | 68.95 ms | 170.43 ms |
| 5,000,000 | 80.15 ms | 334.23 ms | 853.12 ms |
| 10,000,000 | 168.45 ms | 673.32 ms | 1,696.71 ms |

One timed sample per size after prewarming syntax definitions. Timings are
separate operations, not cumulative; terminal output is plain at the default
width. They exclude filesystem loading, process startup, IPC, DOM insertion,
layout and painting. Decimal byte sizes are used.

The synthetic corpus repeats a GFM section with a heading, Unicode paragraph,
link, list, table and Rust code. Its identical code blocks benefit from the
bounded per-render cache (512 entries / 4 MiB highlighted cache). Distinct code
blocks still require separate syntax work; this corpus does not represent all
Markdown workloads. Recognized large code blocks remain highlighted.

Before the cache, the same 10 MB corpus spent about 19.9 s in HTML rendering.
It now spends about 1.7 s. This is a targeted repeated-code improvement, not a
claim that every 10 MB document loads in 1.7 s.

Duplicate heading IDs were another measured bottleneck. A per-base next-suffix
index avoids rescanning all earlier duplicate IDs. The 10,000-repeat regression
is tested for uniqueness; the heading example reproduces scaling independently.

## Executable and native desktop

Twenty fresh CLI processes: `--help` median **5.73 ms**, and plain GFM fixture
read median **5.79 ms**, redirected to `/dev/null`. OS caches were warm. An
initial earlier post-build execution took 207 ms, so the warmed median is not a
first-download/cold-filesystem guarantee.

The native app was launched via macOS LaunchServices (`open -n -W`) with a
local opt-in measurement environment. The ready signal is sent after document
loading, HTML insertion, code/table/outline decoration and initial layout. It
**does not measure first visible paint**. Background WebViews may suspend
animation frames, which makes a two-frame timer inappropriate here.

| Fresh launch with 357-byte GFM fixture | Native entry → DOM/layout ready | Launch command → observed ready |
|---|---:|---:|
| First | 534.77 ms | 1,142.31 ms |
| Second | 558.80 ms | 658.45 ms |
| Third | 539.53 ms | 631.60 ms |

The **500 ms cold desktop startup target is not met**. The first-launch cost
and WebView initialization remain work for later releases. Small-document Rust
kernels meet the practical 100 ms goal; end-to-end startup is a different cost.

## Large native documents

Each size below was opened in a fresh native app, with the OS already warmed:

| Markdown bytes | Native entry → DOM/layout ready | Launch command → observed ready |
|---:|---:|---:|
| 10,000 | 551.19 ms | 647.19 ms |
| 100,000 | 627.15 ms | 732.60 ms |
| 1,000,000 | 1,495.10 ms | 1,587.10 ms |
| 5,000,000 | 5,517.99 ms | 5,607.84 ms |
| 10,000,000 | 11,001.95 ms | 11,089.91 ms |

All five sizes reached layout readiness. This does not establish smooth
scrolling or search latency at every size. The 10 MB corpus expands to about
28.5 MB HTML and 40,817 outline headings; it is an intentionally dense workload.
Rust rendering accounts for only part of its load cost. DOM size, IPC and
outline creation become important. There is no virtualization in v0.1.0.
During the 10 MB load, the native process was observed around 799 MiB RSS;
that was a transient sample, not a measured peak or total process-tree memory.

## Idle resources

After the small GFM fixture, five approximately two-second CPU-time deltas
measured **0% native-process CPU**, at the resolution of macOS `ps` CPU time.
Native RSS was **103–109 MiB**. WebKit's separate content/network/GPU processes
are **excluded**; these values are not total application memory. No continuous
animation or polling is used by the reader; the watcher is event-driven.

The native executable is about 11 MB; app ZIP/DMG about 5 MB. The standalone CLI
is about 1.3 MB. Compressed size does not imply runtime memory use.

## Reproduce locally

```sh
cargo run --release -p marklight-render --example benchmark
cargo run --release -p marklight-core --example headings_cost
python3 scripts/benchmark.py --cli target/release/marklight
```

Close every Marklight instance and build the release app before native tests:

```sh
python3 scripts/benchmark.py \
  --desktop target/release/bundle/macos/Marklight.app/Contents/MacOS/marklight-desktop \
  --large
```

The first command generates size fixtures under ignored `artifacts/`; the
native script writes `startup.json` there and closes the processes it launches.
`MARKLIGHT_BENCH_OUTPUT` is opt-in, controlled by the launching process, and is
never taken from document content. Normal launches write no telemetry or metrics.
Do not run compilation or compression concurrently with timing measurements.
