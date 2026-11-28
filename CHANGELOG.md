# Changelog

## Unreleased

- Lock moves `onemkl` and `onemkl-sys` 0.6.1 to 0.6.5 (the PM's ILP64 bundle; `Cargo.toml` still says `0.6`; this repo does not enable `ilp64`, a library crate must not). Produced with `cargo update -p onemkl --precise 0.6.5` then `cargo update -p bindgen --precise 0.73.2` (the first alone leaves bindgen on syn 2 beside prettyplease 0.3 on syn 3; the second puts it back on syn 3). `cargo test --workspace --locked` passes on Windows (oneAPI 2026.1).

## 0.0.32 - 2026-11-27 (PM-allocated; non-breaking dependency take: onemkl ~0.6, no onemkl type re-exported; README first-use fixes; cpufish stays unpublished)

- `cpufish-onemkl` depends on `onemkl` 0.6 (was 0.5; the lock resolves 0.6.1 with `onemkl-sys` 0.6.1, one copy: `cargo tree -d` lists no onemkl). onemkl 0.6 breaks cell-callback wrappers and makes option/config structs `non_exhaustive` (with builders); this repo uses none (0 hits for `onemkl::df`, `Integrate` and onemkl struct literals; the same grep finds the 7 `onemkl::` references, all `blas::level3::gemm` and `service::*`). Public API unchanged. `Cargo.lock` keeps bindgen 0.73.2 on syn 3 with prettyplease 0.3 (the bump alone flipped it to syn 2; restored). Checked locally on Windows (oneAPI 2026.1, PowerShell): `cargo test --workspace --locked` passes (core 7, aocl 3, onemkl 3, openblas 6, sys 4).
- Docs: README 'Use it' (git dependency until the first publish; crates.io 0.0.0 is a name reservation; the root is the `cpufish` package) and `cargo run --example hello`.

## 0.0.31 - 2026-11-27 (PM-allocated; non-breaking: first-release readiness; cpufish stays unpublished)

- First-release readiness: `examples/hello.rs` and a README example (CI runs it and checks its output); an `msrv (1.95)` CI job; `LICENSE-MIT`/`LICENSE-APACHE`, a README, `keywords` and `categories` in every crate; lock moves `clap` 4.6.10 and `libloading` 0.9.1. Checklist: `docs/FIRST-RELEASE.md`.
- **Yank policy:** a version is yanked only for a correctness bug that silently gives wrong numbers, a security problem, or a licence/provenance error. An ordinary bug is fixed forward in a patch release. Each yank is recorded here with the reason and the fixing version.

## 0.0.30 - 2026-11-27 (PM-allocated; non-breaking dependency take)

- `cpufish-onemkl` depends on `onemkl` 0.5 (was 0.4; the lock resolves 0.5.0 with `onemkl-sys` 0.5.0, one copy). onemkl 0.5.0's only breaking change is `SummaryStats::impute_missing` becoming a free function; this repo does not use it (0 hits for `impute_missing`; the same grep finds `onemkl`). Public API unchanged (no onemkl types are re-exported). Checked locally on Windows (oneAPI 2026.1): `cargo test --workspace` passes (core 7, aocl 3, onemkl 3, openblas 6, sys 4).

## 0.0.29 (allocated by pm-assist 2026-11-27)

- `cpufish-aocl` depends on `aocl-blas`, `aocl-blas-sys` and `aocl-types` 0.2 (were 0.1; the lock resolves 0.2.5 for all five aocl crates, one copy each). Public API unchanged (`aocl_types` is used only inside `cpufish-aocl`). `Cargo.lock` drops bindgen 0.69.5 and its tree (which, with `which`, `itertools`, `rustix`, `windows-sys 0.59`, ...), and bindgen 0.73.2 now pairs syn 3 with prettyplease 0.3. Checked locally on Windows (AOCL 5.3.2 BLIS plus oneAPI `libiomp5md`): `cargo test --workspace` passes (core 7, aocl 3, onemkl 3, openblas 6, sys 4), `clippy -p cpufish-aocl -D warnings` clean.

## 0.0.28 (allocated by pm-assist 2026-11-26)

- `cpufish-onemkl` depends on `onemkl` 0.4 (was 0.3; the lock resolves 0.4.4, with `onemkl-sys` 0.4.4). onemkl's only breaking change in 0.4.0 is `optim::solve_trnls_bounded` taking `TrnlsBounds`; this crate does not use it (it calls `blas::level3::gemm` and `service::*`). 0.4.1 and 0.4.2 are the Windows `c_long` fix (0.4.1 did not compile on Windows). Public API unchanged. `Cargo.lock` drops prettyplease 0.3. Checked locally on Windows: `clippy -D warnings` and the `cpufish-onemkl` tests pass; hosted Linux and Windows CI run the rest.

## 0.0.27 (allocated by pm-assist 2026-11-25)

- CI only: the advisory `portfolio` job pins coderipper `467fd259` (installs CodeRipper 0.48.1, CODERIPPER_VERSION at 467fd259).
- `LICENSE-APACHE` is now the canonical Apache-2.0 text, byte-identical to the sageant copy (11358 bytes, sha256 cfc7749b96f6...); ours was 11282 bytes.

## 0.0.26

- `cpufish-bench` parses its arguments with `clap` 4.6.8 (derive; crates.io) instead of a hand-written loop (OPS-012). Flags unchanged (`--quick`, `--reps N`, `--threads N`, `-h`, `-V`); a bad flag or value still exits 2. Behaviour change: `--threads 0` is now rejected (it was passed to `set_threads`). `cpufish-bench` is `publish = false`, so no public crate API changes.

## 0.0.25

- Docs: README rewritten to match the crates and hosted CI, and it now says the provider jobs run on every PR but are not among the 7 required checks on `main` (audited against branch protection and `ci.yml`, 2026-11-19). ROADMAP rows 3, 5, 6 and 8 updated to hosted CI and onemkl 0.3.
- `cpufish-bench`: `--reps 0` is now a usage error (exit 2); it used to panic taking the median of an empty list. Public API unchanged.

## 0.0.24

- `cpufish-onemkl` depends on `onemkl` 0.3 (was 0.2; ciresnave/onemkl#3 fixed in onemkl 0.3.0). The Linux `RUSTFLAGS` link-line workaround is dropped from the hosted `onemkl-linux` job and its tests pass without it (proof: run 44434675890, lp64 / link-dynamic / threading-sequential only; OpenMP, TBB and ILP64 untested). Public API unchanged: `onemkl` types appear only in private `use` lines, the public surface is the `OneMkl` provider struct. `Cargo.lock` relocks bindgen 0.73.2 with prettyplease 0.3.

## 0.0.23

- Deleted the vestigial self-hosted `.github/workflows/aocl.yml` (SUP-017; hosted jobs cover the providers, benchmarks run by hand). `cpufish-bench` gains `--help`/`--version` and exits 2 with usage on a bad flag or missing number instead of panicking (OPS-011).
- CI only: the advisory `portfolio` job moves to the shared workflow at coderipper `c72a6e0` (installs CodeRipper 0.47.5). `tools/migrate-rule-ids.py` finds no `.coderipper.toml` and no rule names in this repo, so nothing to migrate.

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
