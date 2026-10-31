# Changelog

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
