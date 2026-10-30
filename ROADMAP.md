# CPUFish roadmap

Statuses: TODO, DOING, DONE, BLOCKED. Last updated 2026-10-30.

| # | Row | Status | Notes |
|---|-----|--------|-------|
| 0 | `ROADMAP.md`, CI (fmt, clippy, test, doc, cargo-deny, portfolio), stub crate | DONE (PR #1) | branch protection on, 7 checks required |
| 1 | Survey and design note: Fuel's CPU path, AOCL, oneMKL CPU, OpenBLAS/BLIS/Accelerate/ARM PL, SIMD kernels; Windows+Linux, x86-64+aarch64; licences. Options, recommendation, no code | DONE (PR #2) | docs only |
| 2 | Design the vendor-neutral CPU layer with the Fuel lane (run-time provider selection) | DONE (PR #3; Fuel lane reviewed 2026-10-30) | after row 1; no edits to Fuel's repo |
| 3 | Take over `ciresnave/aocl`: adopt its ROADMAP, keep Fuel's `aocl-blas`/`aocl-types` 0.1 dependency working | DOING | aocl#4 (ROADMAP) and #5 (cargo-deny) merged; remaining rows in aocl/ROADMAP.md | SemVer; coordinate with the Fuel lane before any breaking release |
| 4 | `cpufish` core crate (S1): Provider, Registry, portable provider, conformance suite | DONE (PR #4) | RUSTSEC-2024-0436 (`paste` via `gemm`) acknowledged in deny.toml per PM ruling 2026-10-30 |
| 5 | `cpufish-aocl` (S2): design in `docs/S2-AOCL-DESIGN.md`, then code | DOING (design PR) | link-time provider over `aocl-blas`; tests need the self-hosted runner |
| 6 | `cpufish-onemkl` (S3), bench harness (S4), OpenBLAS (S5), loader `cpufish-sys` (option B) | TODO | after S2 |

Out of scope: Unpopped's `unpopped-cpu-c` emitter (Unpopped's unless CireSnave rules); `onemkl`/`onemkl-sys` (Bonefish lane; consume from crates.io).
