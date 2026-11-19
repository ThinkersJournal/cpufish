# CPUFish roadmap

Statuses: TODO, DOING, DONE, BLOCKED. Last updated 2026-11-19.

| # | Row | Status | Notes |
|---|-----|--------|-------|
| 0 | `ROADMAP.md`, CI (fmt, clippy, test, doc, cargo-deny, portfolio), stub crate | DONE (PR #1) | branch protection on, 7 checks required |
| 1 | Survey and design note: Fuel's CPU path, AOCL, oneMKL CPU, OpenBLAS/BLIS/Accelerate/ARM PL, SIMD kernels; Windows+Linux, x86-64+aarch64; licences. Options, recommendation, no code | DONE (PR #2) | docs only |
| 2 | Design the vendor-neutral CPU layer with the Fuel lane (run-time provider selection) | DONE (PR #3; Fuel lane reviewed 2026-10-30) | after row 1; no edits to Fuel's repo |
| 3 | Take over `ciresnave/aocl`: adopt its ROADMAP, keep Fuel's `aocl-blas`/`aocl-types` 0.1 dependency working | DOING | aocl#4 (ROADMAP) and #5 (cargo-deny) merged; remaining rows in aocl/ROADMAP.md; SemVer, coordinate with the Fuel lane before any breaking release |
| 4 | `cpufish` core crate (S1): Provider, Registry, portable provider, conformance suite | DONE (PR #4) | RUSTSEC-2024-0436 (`paste` via `gemm`) acknowledged in deny.toml per PM ruling 2026-10-30 |
| 5 | `cpufish-aocl` (S2): design in `docs/S2-AOCL-DESIGN.md`, then code | DONE (design PR #5; code PR with 0.0.5) | link-time provider over `aocl-blas`; AOCL tests run in hosted CI on Linux and Windows against amd/blis 5.3.2 built from source, not AMD's installer (`docs/NATIVE-LIBS.md` "Hosted CI"), and pass locally (2026-11-16, Windows). Tests: `aocl_passes_conformance`, `aocl_probe_succeeds`, `aocl_reports_version_threads_and_isa` |
| 6 | `cpufish-onemkl` (S3): link-time provider over `onemkl` 0.3 | DONE (0.0.6; onemkl 0.3 since 0.0.24) | tests run in hosted CI (Linux apt, Windows PyPI `mkl`) and pass locally. Linux link needs no RUSTFLAGS since onemkl 0.3 (onemkl#3; lp64, link-dynamic, threading-sequential only; OpenMP, TBB and ILP64 untested): `onemkl_passes_conformance`, `onemkl_probe_succeeds`, `onemkl_reports_version_threads_and_isa` |
| 7 | Bench harness (S4) | DONE (0.0.7) | `crates/cpufish-bench`; timings are NOT a ranking (shared, loaded box); numbers withheld from docs (PM ruling); shapes are a superset, Fuel has no fixed list |
| 8 | OpenBLAS (S5), loader `cpufish-sys` (option B), re-measure S4 on a quiet box | design in `docs/S5-OPENBLAS-DESIGN.md` (0.0.11); provider + loader DONE (0.0.14), bench `openblas` feature DONE (0.0.18); OpenBLAS tests run in hosted CI (Linux, Windows); S4 re-measure PENDING: benchmarks run by hand on a quiet machine, not in CI and no runner (CireSnave 2026-11-01), gated until the box is quiet | oneMKL measured on an AMD CPU is a valid test of the provider; conclude nothing about Intel hardware (PM 2026-10-30). Threads: `--threads N` + `Provider::set_threads` DONE (0.0.8); Fuel runs library defaults, so record those separately |

Out of scope: Unpopped's `unpopped-cpu-c` emitter (Unpopped's unless CireSnave rules); `onemkl`/`onemkl-sys` (Bonefish lane; consume from crates.io).
