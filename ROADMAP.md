# CPUFish roadmap

Statuses: TODO, DOING, DONE, BLOCKED. Last updated 2026-10-30.

| # | Row | Status | Notes |
|---|-----|--------|-------|
| 0 | `ROADMAP.md`, CI (fmt, clippy, test, doc, cargo-deny, portfolio), stub crate | DOING | first PR; then ask the PM to require the checks |
| 1 | Survey and design note: Fuel's CPU path, AOCL, oneMKL CPU, OpenBLAS/BLIS/Accelerate/ARM PL, SIMD kernels; Windows+Linux, x86-64+aarch64; licences. Options, recommendation, no code | TODO | docs only |
| 2 | Design the vendor-neutral CPU layer with the Fuel lane (run-time provider selection) | TODO | after row 1; no edits to Fuel's repo |
| 3 | Take over `ciresnave/aocl`: adopt its ROADMAP, keep Fuel's `aocl-blas`/`aocl-types` 0.1 dependency working | TODO | SemVer; coordinate with the Fuel lane before any breaking release |
| 4 | `cpufish-sys` / `cpufish` implementation | BLOCKED | on rows 1-2 |

Out of scope: Unpopped's `unpopped-cpu-c` emitter (Unpopped's unless CireSnave rules); `onemkl`/`onemkl-sys` (Bonefish lane; consume from crates.io).
