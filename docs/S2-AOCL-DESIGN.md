# S2 design: `cpufish-aocl` (design only, no code in this PR)

Facts measured 2026-10-30: `ciresnave/aocl` `origin/main` 987f623 (workspace 0.1.4), `ciresnave/fuel` `origin/main`,
`ThinkersJournal/cpufish` `origin/main` d4f7ee5 (S1 merged), local AOCL 5.3.2 install on the author's Windows box.

## What the AOCL crates give us

- `aocl-blas` exposes `<f32 as aocl_blas::Scalar>::gemm(Layout, Trans, Trans, m, n, k, alpha, a, lda, b, ldb, beta, c, ldc)`
  (trait method, `src/lib.rs:596`, impls over `cblas_sgemm`/`cblas_dgemm`), which takes explicit leading dimensions
  and has a shape checker (`check_gemm_shapes`, `:419`). The free function `aocl_blas::gemm` (`:3043`) is
  tightly-packed only, so it cannot carry `GemmArgs::lda/ldb/ldc`. The adapter must use the trait method.
  `aocl_types::{Layout, Trans}` map one to one onto `cpufish::Trans` plus `RowMajor`.
- ⚠️ **`aocl-blas-sys` links at build time** (`build.rs` emits `rustc-link-lib`; dynamic by default, `links = "amd-blis"`;
  needs `AOCL_ROOT` and libclang for bindgen). On Windows a missing `AOCL-LibBlis-Win-MT-dll.dll` therefore stops the
  *process* at load, before any `probe()` can run. Fuel's `dll_path::ensure_loadable` only adds the install dir to `PATH`
  for a dll that exists. So "unavailable at run time" is not achievable with `aocl-blas` as it stands; the survey's
  "missing library is unavailable, never a link error" goal holds only for providers that load their library themselves.

## Options

- **A. Link-time provider (recommended for S2).** `cpufish-aocl` depends on `aocl-blas`; a program that includes it
  requires AOCL at build and start. Run-time selection still works among the providers *present in the binary*
  (portable + aocl + onemkl ...), which is exactly what A/B-testing vendor stacks needs. Smallest step; reuses
  published, Fuel-proven crates; `probe()` is the known-answer 2x2 gemm plus a version read.
- **B. Loader provider.** `cpufish-sys` loads `libblis`/`cblas_sgemm` with `libloading`, so a missing library is a
  recoverable `ProbeError::Unavailable` and one binary runs everywhere. Cost: a second FFI surface for the subset
  CPUFish uses, symbol-name differences per vendor, our own Windows search-path logic. This is the long-term reason
  `cpufish-sys` is reserved.
- **C. Delay-load the import library (MSVC `/DELAYLOAD`).** Keeps `aocl-blas`, gets run-time absence on Windows only;
  no Linux equivalent besides option B. Not pursued.

Recommendation: A now, B later (S5+, one `cpufish-sys` loader shared by the BLAS-ABI vendors AOCL, OpenBLAS, and
oneMKL's CBLAS entry points) once a single binary must run without any vendor installed. Both fit the S1 `Provider`
trait unchanged, so choosing A forecloses nothing.

## Layout and CI

- Convert the repo to a Cargo workspace: `cpufish` (core, unchanged API) and `crates/cpufish-aocl`. The aocl crate is
  **not** in `default-members`, so hosted CI (`cargo test`, clippy, fmt, doc, deny) never needs AOCL.
- `cpufish-aocl` tests *are* the conformance suite (`cpufish::conformance::check_provider(&Aocl::new())`) plus a
  version/config test. They need AOCL, so they run on the self-hosted Windows runner under the standing policy
  (5.1g/5.1h): push to main, schedule, dispatch; same-repo PRs only behind the environment gate; never fork code;
  never `#[ignore]`. This repo is PUBLIC, so the runner job is non-required for fork PRs and the post-merge run on main
  is the device check. The runner must be registered for this repo (PM fetches the token); until it is, the tests are
  **UNVERIFIED in CI** and I state so, though I can run them locally (AOCL 5.3.2 is installed here).
- Linux AOCL (x86-64) is untested: no Linux runner or box is known (UNVERIFIED).
- `cargo deny` on the workspace needs `aocl-blas` 0.1 -> published 0.1.0 only (0.1.1-0.1.4 unpublished); the crate
  must depend on `aocl-blas = "0.1"` from crates.io, never a path (portfolio rule). Publishing 0.1.4 is the PM's call
  and is not required for S2.

## `Aocl` provider behaviour

- `id() = "aocl"`; `version()` from AOCL's version query (aocl-utils / `bli_info_get_version_str`, to confirm in the
  implementation PR); `capabilities()`: Gemm F32, F64.
- `effective_config()` reports BLIS's thread count (`bli_thread_get_num_threads`, or `BLIS_NUM_THREADS`/`OMP_NUM_THREADS`
  as read at probe) and the selected micro-architecture (`bli_arch_string`), because Fuel asked for effective threads
  and ISA. Names are to be verified against the installed headers before use; if a query is unavailable the field is
  `None` rather than guessed.
- `probe()` runs the same 2x2 known-answer check as Fuel's `probe_aocl_loadable`; no global state is changed. Thread
  count is read, not set. A `with_threads(n)` constructor is deferred (setting BLIS threads is process-global).
- Errors from `aocl_error::Error` map to `cpufish::Error::Backend { provider: "aocl", .. }`; `GemmArgs::validate()` runs
  first so the safe wrapper's own checks are a second line, not the only one.

## Open questions

1. PM: register `ThinkersJournal/cpufish` with the self-hosted runner (needed for any AOCL/oneMKL test in CI).
2. Fuel lane: is link-time provider selection acceptable for Fuel's first A/B runs (Fuel already links `aocl-blas`)?
3. aocl lane work this implies: none for A. For B, a BLAS-ABI loader is new code in `cpufish-sys`, not in `aocl`.
