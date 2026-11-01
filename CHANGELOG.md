# Changelog

## 0.0.22

- CI: two more hosted jobs, Windows counterparts of 0.0.21 (proof run 41351554183): `onemkl-windows` (Intel's PyPI `mkl` packages, pinned 2026.2.0) and `aocl-windows` (amd/blis 5.3.2 at a pinned commit, CMake + ClangCL). HANDOFF rewritten: board 364 is archived and benchmarks are run by hand on a quiet machine (CireSnave), so no runner is awaited.

## 0.0.21

- CI: three hosted jobs run the provider tests against the real libraries (board 364, proof run 41348787718): `openblas-windows` (release zip, SHA-256 pinned), `onemkl-linux` (Intel apt repo, key pinned by fingerprint), `aocl-linux` (amd/blis 5.3.2 built from source at a pinned commit). Terms recorded in `docs/NATIVE-LIBS.md` "Hosted CI". Windows AOCL/oneMKL and the S4 bench stay on the local runner (`aocl.yml`).

## 0.0.20

- Docs only: HANDOFF records the exact board 364 wait line and the current state. No code change.

## 0.0.19

- CI only: `aocl.yml` load log reads CPU busy % from CIM instead of `Get-Counter`, which failed for the runner account in run 41228682257 and failed the step. Discovery found AOCL, libiomp5md and libclang there; `cargo` was not on that account's PATH (not fixed here: toolchain install is the PM's).

## 0.0.18

- `cpufish-bench` gains an `openblas` feature (run-time loaded, so it builds anywhere and the provider is skipped when no library is found). The hosted `openblas-linux` job now also runs `bench --quick` with it and fails unless `openblas` appears in the table.

## 0.0.17

- Docs only: `docs/NATIVE-LIBS.md` records the Windows LP64 OpenBLAS 0.3.34 conformance run (5 passed, standard user), and adds test `loader_reports_the_expected_int_width` (runs only with `CPUFISH_OPENBLAS_EXPECT_WIDTH` and `CPUFISH_OPENBLAS` set). Windows x64 verified at both widths in the PM's run (width asserted, with a failing control); ILP64 elsewhere stays UNVERIFIED.

## 0.0.16

- New `docs/NATIVE-LIBS.md`: the native-library convention (one admin-owned read-only location per library and version, pinned version and zip SHA-256, never on `PATH`) for AOCL, oneMKL and OpenBLAS, and an elevated install procedure.
- New `tools/install-openblas.ps1`: downloads the pinned OpenBLAS 0.3.34 zips, checks both hashes before extracting, installs to `%ProgramFiles%\OpenBLAS\0.3.34\{lp64,ilp64}`, sets the ACL and reads it back; `-VerifyOnly` checks hashes without installing. Not run as an install by this lane.
- `cpufish-openblas` search order: on Windows the default is now that install directory (no bare names, no `PATH`) and `CPUFISH_OPENBLAS` must be an absolute path; Linux/macOS unchanged. Behaviour change on Windows only; the crate is `publish = false`.

## 0.0.15

- CI only: `aocl.yml` (dispatch only) now also runs the S4 bench in the same job as the `cpufish-aocl` tests, because the self-hosted runner is ephemeral (one job per registration). Steps: machine and CPU-load log, optional oneMKL discovery, bench build (aocl, and aocl+onemkl when oneMKL is found), then the bench at library-default threads, `--threads 1` and `--threads 4`, written to the job summary. Independent steps run `if: always()`.

## 0.0.14

- CI only: new `openblas-linux` job installs `libopenblas0` on `ubuntu-latest` and runs `cpufish-openblas` clippy and tests with `CPUFISH_OPENBLAS_REQUIRED=1`, so conformance runs against a real library there. Windows stays UNVERIFIED.

## 0.0.13

- New `crates/cpufish-sys` (default member): a `libloading` run-time loader for CBLAS-ABI libraries with a safe, bounds-checked gemm; reads LP64/ILP64 from OpenBLAS's config string. New `crates/cpufish-openblas` (S5): `OpenBlas` provider over it, gemm f32/f64, `set_threads`, effective threads and kernel name; an absent library is `ProbeError::Unavailable`. Library found via `with_path`, `CPUFISH_OPENBLAS`, then platform default names. Not a default member. Both `publish = false`. Conformance against a real OpenBLAS is UNVERIFIED at this version.

## 0.0.12

- CI only: the advisory `portfolio` job now pins coderipper's `portfolio.yml` at `db626e88` (coderipper 0.17.3, whose failure causes include the linker line) instead of `30e012b`.

## 0.0.11

- Docs only: `docs/S5-OPENBLAS-DESIGN.md`, the design for the `cpufish-sys` run-time loader and a `cpufish-openblas` provider (ROADMAP row 8). No code change.

## 0.0.10

- New `.github/workflows/aocl.yml`: runs the `cpufish-aocl` tests on a self-hosted runner labelled `[self-hosted, Windows, X64, aocl]`. `workflow_dispatch` only, no secrets, no `pull_request` trigger. It locates AOCL, `libiomp5md.dll` and `libclang.dll` itself and fails loudly if any is missing.

## 0.0.9

- Hygiene, no behaviour change: CI cargo steps run `--locked` (SUP-001); the two `allow` attributes carry `reason`; every `.rs` file starts with an SPDX identifier.

## 0.0.8

- `Provider::set_threads(n)` (defaulted, so not breaking): sets the library thread count, process-global, to be called once at startup. Implemented for `aocl` (`bli_thread_set_num_threads`), `onemkl` (`MKL_Set_Num_Threads`) and `portable` (accepts only 1).
- `cpufish-bench --threads N` applies it before any gemm and prints the requested count beside each provider's effective config. Without it, providers run at library defaults as Fuel does.
## 0.0.7

- New `crates/cpufish-bench` (S4): `cpufish-bench [--quick] [--reps N]` compares installed providers on Fuel-style gemm shapes (f32, NoTrans/NoTrans, alpha=1, beta=0, one gemm per slice) and checks each against `portable`. Vendor providers are cargo features `aocl`, `onemkl`, `onemkl-openmp`. `publish = false`.

## 0.0.6

- New `crates/cpufish-onemkl` (S3): `OneMkl` provider over the crates.io `onemkl` 0.2 cblas gemm f32/f64; workspace member, not a default member (needs oneMKL). Cargo features pick the threading layer (default sequential; `threading-intel-openmp`). `publish = false`.

## 0.0.5

- Cargo workspace; new `crates/cpufish-aocl` (S2): `Aocl` provider over `aocl-blas` 0.1 gemm f32/f64, not a default member (needs AOCL). `publish = false`.
- Core crate unchanged.

## 0.0.4

- Add `docs/S2-AOCL-DESIGN.md`, the `cpufish-aocl` design (docs only).

## 0.0.3

- Core crate (S1): `Provider` trait, `Registry`, pure-Rust `portable` provider (gemm f32/f64), shared `conformance` suite. Still `publish = false`.

## 0.0.2

- Add `docs/PROVIDER-DESIGN.md`, the provider trait sketch (ROADMAP row 2). Docs only.

## 0.0.1

- Add `docs/SURVEY.md`, the survey and design note (ROADMAP row 1). Docs only; the crate is still a placeholder.

## 0.0.0

- Placeholder crate, ROADMAP, CI.
