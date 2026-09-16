# Performance — measured, not inferred

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
