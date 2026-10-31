# S5 design: `cpufish-sys` loader and an OpenBLAS provider (design only, no code in this PR)

Facts measured 2026-10-31: crates.io (`openblas-src` 0.10.16, Apache-2.0/MIT, MSRV 1.71.1; `cblas-sys` 0.3.0,
Apache-2.0 OR MIT; `libloading` 0.9.0, ISC, MSRV 1.88.0), OpenBLAS `develop` `cblas.h`, OpenBLAS release v0.3.34
(2026-07-16, BSD-3-Clause), `ThinkersJournal/cpufish` `origin/main` 54d02f8 (0.0.10). Nothing here was built.

## Why this step

S2/S3 are link-time providers (`docs/S2-AOCL-DESIGN.md`, option A): a program containing them needs the vendor
library to start. S5 delivers the other half of the survey goal, "a missing library is unavailable, never a
link error", and adds a third vendor stack for A/B runs. One loader serves every vendor that exposes the CBLAS ABI.

## What OpenBLAS exposes (from `cblas.h`)

- `cblas_sgemm` / `cblas_dgemm`: same signature as BLIS and MKL CBLAS.
- Introspection we need for `effective_config()`: `openblas_get_num_threads`, `openblas_get_corename` (the selected
  kernel set, our "ISA"), `openblas_get_config` (build string, includes the version). Setter:
  `openblas_set_num_threads` (process-global, like BLIS).
- ⚠️ **`blasint` is `int` (LP64) or 64-bit (ILP64) depending on the build**; the official Windows zips ship both
  (`OpenBLAS-0.3.34-x64.zip`, `...-x64-64.zip`) and ILP64 builds may name their symbols differently (UNVERIFIED: check the exports of the `-64` zip before relying on a name). The loader
  must pick one width explicitly (LP64 first; ILP64 is a later cargo feature) and reject a library of the other width
  rather than call it with the wrong argument size.
- Windows on Arm (`woa64`) builds exist; that is untested here.

## Options

- **A. `openblas-src` + `cblas-sys` (link-time).** Published, maintained, can build OpenBLAS from source or link a
  system copy. Same limitation as S2: absence stops the process (Windows) or fails the link. Gains a provider quickly,
  forecloses nothing, does not deliver run-time absence.
- **B. `cpufish-sys` loader with `libloading` (recommended).** `cpufish-sys` defines one small `BlasAbi` table
  (`sgemm`, `dgemm`, optional `get_num_threads`, `set_num_threads`, `corename`, `config`) resolved with
  `libloading::Library::new` + `get`. A missing library or symbol becomes `ProbeError::Unavailable`; one binary then
  runs everywhere. Cost: our own `unsafe` FFI for ~6 symbols, our own search-path logic, `blasint` width handling.
  `libloading` is ISC, which `deny.toml` already allows. MSRV 1.88 is below our 1.95.
- **C. Both.** An `openblas-src` provider for build-from-source users plus the loader. Rejected for now: two ways to
  reach the same library doubles the test matrix.

Recommendation: **B**, OpenBLAS first, then (separately, measured) re-point `cpufish-aocl`/`cpufish-onemkl` at the
loader only if a single no-vendor-installed binary is actually wanted. They keep working unchanged meanwhile, and
Bonefish owns the onemkl crates (we consume, never edit), so a loader-based MKL provider would be a new crate of ours,
not a change to theirs. This also answers the standing portfolio-sweep failure that `cpufish-aocl`/`-onemkl` cannot
link on hosted Linux: a loader-only `cpufish-openblas` could be a default member and run on hosted CI if the runner
image has `libopenblas` (`apt install libopenblas-dev`, to be verified, not assumed).

## Layout

- `crates/cpufish-sys`: owns the `unsafe`. Public API is safe: `Library::open(candidates) -> Result<_, LoadError>`,
  `Library::sgemm/dgemm(...)` taking `GemmArgs`-shaped plain values and bounds-checking before the call.
  `#![deny(unsafe_op_in_unsafe_fn)]`, every `unsafe` block carries a SAFETY comment, symbol-presence tests.
  The crate name is already reserved at 0.0.0 on crates.io.
- `crates/cpufish-openblas`: `OpenBlas` provider over `cpufish-sys`; same conformance suite as S2/S3
  (`cpufish::conformance::check_provider`); `probe()` = open + 2x2 known-answer gemm; `version()` parsed from
  `openblas_get_config` (field is `None` if the string has no recognisable version, never guessed).
- Search order: explicit path argument, then env `CPUFISH_OPENBLAS`, then the platform default names
  (`libopenblas.so.0`, `libopenblas.dylib`, `libopenblas.dll`, `openblas.dll`). On Windows we do not alter `PATH`.
- Threads: `set_threads` calls `openblas_set_num_threads` when the symbol exists, otherwise returns `Unsupported`;
  `effective_config()` reports `openblas_get_num_threads` and `openblas_get_corename`.
- ⚠️ OpenBLAS threading flavours (pthreads, OpenMP, serial) change behaviour; the config string says which, and the S4
  bench records it beside the timing.

## Testing and CI

- Linux hosted: install `libopenblas0` on the runner image and run the provider tests for real (positive control: the
  same job asserts the library really was found, so a skipped load cannot pass as green).
- Windows: the self-hosted runner (`aocl.yml` pattern, label `aocl` box) with an unpacked official zip; the
  `CPUFISH_OPENBLAS` env var points at it. UNVERIFIED until the runner is registered.
- Tests never `#[ignore]` (standing rule); an absent library is a probe result under test, not a skip.
- Loader unit tests use a tiny test-only shared library or a deliberately missing name to prove `Unavailable`.

## Versions and process

Docs-only PR, next version 0.0.11 (PM allocates at gate). The implementation is two further PRs, one at a time: (1)
`cpufish-sys` plus `cpufish-openblas`, (2) S4 bench and CI wiring. LAPACK/FFT/vector-math ops (`PROVIDER-DESIGN.md`
S5 tail) are a separate design after gemm is measured.

## Open questions

1. PM: is a hosted Linux job with `libopenblas0` acceptable, or must every vendor-library test sit on the self-hosted
   runner? (Default I will follow: hosted if the package installs from the image's own apt, runner otherwise.)
2. Fuel lane: is LP64 sufficient for Fuel's shapes (it uses `usize` dims that fit `i32` today)? ILP64 can wait.
3. Not yet checked: OpenBLAS licence texts of the Windows zip contents (bundled `libgfortran`/`libgcc` runtime
   exceptions), Accelerate and Arm PL via the same loader (they use the CBLAS ABI, but their licences are UNVERIFIED
   in `SURVEY.md`).
