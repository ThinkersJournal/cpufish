# Changelog

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
