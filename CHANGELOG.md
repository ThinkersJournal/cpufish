# Changelog

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
