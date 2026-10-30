# CPUFish survey and design note (row 1; options first, no code)

Measured 2026-10-30. Fuel facts are at `origin/main` of `ciresnave/fuel` (fetched that day); crates.io and GitHub
facts are from the live APIs that day; local-install facts are from the author's Windows box.
Items marked **UNVERIFIED** were not checked against a primary source and must be before they drive a decision.

## 1. Fuel's CPU path today

- `fuel-cpu-backend`: portable byte-kernel CPU backend (elementwise, reduce, norm, rope, attention, conv, quant matmul, ...).
  Matmul through the `gemm` crate; optional features `mkl` (via `intel-mkl-src`) and `accelerate` (via `accelerate-src`).
  Its own inventory: `fuel-dispatch/kernel-contracts/_inventory/cpu.md`.
- `fuel-cpu-kernels`: SIMD primitives (`avx.rs`, `neon.rs`, erf, philox RNG); deps `half`, `float8`, `libm`.
- `fuel-mkl-cpu-backend` (dep `onemkl` 0.2) and `fuel-aocl-cpu-backend` (deps `aocl-blas`/`aocl-types` 0.1): sibling
  backends that register **two kernels each, both F32**: MatMul and Conv2D (conv via the shared `fuel-conv` im2col+gemm),
  as alternatives on the binding table at `BackendId::Cpu`, tagged `kernel_source` `"mkl"` / `"aocl"`. Availability is a
  runtime 2x2 sgemm probe (`probe_aocl_loadable`); the Judge profiles alternatives per (op, dtype, size class).
  Inventory: `_inventory/mkl-aocl.md`.
- Consequence: vendor libraries today cover only F32 gemm. Everything else (and all other dtypes) is Fuel-portable code.
  That is the gap a vendor-neutral layer can fill: the same op set offered by each vendor, so each stack is tested on
  identical work.

## 2. What each vendor stack offers on CPU

| Stack | Components relevant to CPU math | Licence (verified at) | Platforms |
|---|---|---|---|
| AMD AOCL 5.3 (newest, 2026-05-18 per amd.com; local install is BLIS 5.3.2) | BLAS (BLIS), LAPACK (libFLAME), FFTW, LibM, Sparse, ScaLAPACK, RNG, SecureRNG, Utils, plus compression/crypto/DA | BLIS & LibM: BSD-3-Clause (AMD `LICENSE` read, 2026-10-30); Sparse: MIT (AMD `LICENSE.md` read); libFLAME/FFTW: own LICENSE files, **not yet read**; binary installer is under AMD's EULA | Windows + Linux, x86-64 only (AMD-tuned, runs on any x86-64) |
| Intel oneMKL (local 2026.1) | BLAS, LAPACK, FFT, VM (vector math), RNG, sparse, PARDISO, ... | Intel Simplified Software License; redistribution permitted per Intel's licence FAQ (not the licence text; read it before shipping) | Windows + Linux, x86-64; macOS dropped on Intel's side **UNVERIFIED** |
| OpenBLAS | BLAS, LAPACK | BSD-3-Clause (GitHub API, 2026-10-30; active, pushed same day) | Windows, Linux, macOS; x86-64, aarch64 and more |
| BLIS (upstream flame/blis) | BLAS + BLIS typed API | GitHub reports NOASSERTION; upstream is BSD-3 per project docs **UNVERIFIED** | as above |
| Apple Accelerate | BLAS, LAPACK, vDSP, vForce, BNNS | Apple SDK terms, **UNVERIFIED** | macOS/iOS, aarch64 |
| Arm Performance Libraries | BLAS, LAPACK, FFT, sparse, libamath | **UNVERIFIED** (licence and Windows-on-Arm availability not checked) | aarch64 Linux/Windows/macOS |
| Pure-Rust: `gemm` 0.19, `faer` 0.24, `matrixmultiply` 0.3, `pulp` 0.22 | gemm, dense factorisations, SIMD dispatch | permissive (not read) | everywhere |

Rust binding landscape (crates.io, 2026-10-30): `onemkl` is ours (Bonefish lane; consume only). `openblas-src` 0.10.16
and `cblas-sys` 0.3.0 are maintained; `blis-src` 0.2.2 (2024), `intel-mkl-src` 0.8.1 (2022) and `accelerate-src` 0.3.2
(2018) are stale, and `intel-mkl-src`/`accelerate-src` are what `fuel-cpu-backend` uses behind features today.

## 3. The `ciresnave/aocl` repo (to take over)

29 crates in 12 sys/safe pairs (BLAS, LAPACK, FFT, LibM, Sparse, RNG, SecureRNG, Utils, Compression, Crypto,
Data Analytics, ScaLAPACK) plus `aocl-build`, `aocl-error`, `aocl-types` and the `aocl-sys`/`aocl` umbrellas; 0.1.2 on
`origin/main` (7fbe68f); no `ROADMAP.md` exists (row 3 creates one). Requires an AOCL install and libclang at build
time (bindgen). AOCL 5.3.2 is installed locally, so the wrappers can be tested here; `aocl-scalapack` is explicitly
"under construction".

## 4. Design options for the vendor-neutral layer

- **A. Trait + per-vendor provider crates, run-time selection (recommended).** `cpufish` defines the op traits (gemm,
  batched gemm, LAPACK subset, FFT, vector math, RNG, sparse) over typed buffers and a `Provider` registry; one thin
  crate per vendor (`cpufish-aocl`, `cpufish-onemkl`, `cpufish-openblas`, `cpufish-accelerate`, `cpufish-armpl`,
  `cpufish-portable`) built on the existing sys/safe crates. Each provider probes its library at run time (dlopen via
  the existing probe pattern) and reports capabilities per (op, dtype); Fuel's Judge then picks per (op, dtype, size).
  Same work, every stack: exactly the stated goal. Cost: a capability matrix to define and keep honest.
- **B. Cargo-feature selection at compile time.** Simpler, but a build carries one vendor, so one binary cannot compare
  stacks; rejected for the stated goal.
- **C. Adopt a BLAS-ABI shim (link one `libblas`, swap by loader).** Cheap, covers only BLAS/LAPACK, cannot A/B
  vendors in one process, nothing for FFT/VM/RNG. Rejected.

## 5. Recommendation

Option A, staged: (1) a `cpufish` core crate with the trait surface and capability probe, starting with F32/F64 gemm and
batched gemm since that is what Fuel consumes; (2) `cpufish-aocl` over `aocl-blas`/`aocl-types` (and reuse the same
crate set for LAPACK/FFT/LibM later); (3) `cpufish-onemkl` over crates.io `onemkl`; (4) OpenBLAS provider via
`openblas-src`/`cblas-sys`; (5) Apple Accelerate and Arm PL providers once their licences and Rust bindings are verified
and a test box exists. Half-precision gemm and quantised matmul stay Fuel-portable until a vendor offers them.
`cpufish-sys` stays reserved for any raw FFI the project needs that no existing `*-sys` crate covers.

## 6. Open questions (for the Fuel lane and PM)

1. Does Fuel want CPUFish to replace `fuel-aocl-cpu-backend`/`fuel-mkl-cpu-backend`, or sit beneath them? (Recommend beneath, then migrate; the Fuel lane owns that repo.)
2. Is `fuel-conv`'s im2col+gemm to stay in Fuel, or move into CPUFish as shared code?
3. aarch64 test hardware: none known on this box (x86-64 only measured); GitHub macOS/aarch64 runners cover Accelerate only.
4. Windows AOCL/oneMKL runtime tests need self-hosted runners (portfolio rule 5.1g/5.1h); the box has both installed.
