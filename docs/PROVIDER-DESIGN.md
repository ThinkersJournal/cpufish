# CPUFish provider trait sketch (ROADMAP row 2; design only, no code merged)

Builds on `docs/SURVEY.md` option A. Facts measured 2026-10-30: `aocl_blas::gemm<T: Scalar>(Trans, Trans, m, n, k, alpha,
&[T], &[T], beta, &mut [T])` (row-major, leading dims derived; `aocl-blas` `src/lib.rs:3043`, aocl 0.1.3);
`onemkl::blas::level3::gemm<T: BlasScalar>(Transpose, Transpose, alpha, &MatrixRef, &MatrixRef, beta, &mut MatrixMut)`
(`onemkl/src/blas/level3.rs:23`); Fuel's wrappers call one gemm per batch slot, NoTrans/NoTrans, alpha=1, beta=0, F32 only
(`fuel-aocl-cpu-backend/src/binding_table.rs`). The two vendor signatures differ in shape, which is what the trait absorbs.

## Goals and non-goals

- Same op, same inputs, every vendor; one process can hold several providers and run each on identical work.
- Providers are discovered and probed at run time; a missing library is "unavailable", never a link error for the program.
- Non-goals: owning Fuel's kernel registry, tensors, or scheduling (Fuel's Judge picks); duplicating vendor features.

## Shape of the API (names provisional)

```rust
pub enum Dtype { F32, F64, C32, C64 }           // half types: not offered by vendors yet (survey 5)
pub enum Op { Gemm, GemmBatched, /* later: Getrf, Gesv, Fft, VmExp, ... */ }

pub struct Capability { pub op: Op, pub dtype: Dtype, pub notes: &'static str }

pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;                // "aocl", "onemkl", "openblas", "portable"
    fn version(&self) -> Option<String>;          // library version if the library reports it
    fn probe(&self) -> Result<(), ProbeError>;    // loads the library, runs a 2x2 known-answer gemm
    fn capabilities(&self) -> &[Capability];      // what it claims, per (op, dtype)
    fn effective_config(&self) -> Config;        // threads + ISA actually in force
    fn gemm_f32(&self, a: &GemmArgs<f32>) -> Result<(), Error>;   // typed entry points, default = Unsupported
    fn gemm_f64(&self, a: &GemmArgs<f64>) -> Result<(), Error>;
}
pub struct GemmArgs<'a, T> { trans_a: Trans, trans_b: Trans, m: usize, n: usize, k: usize,
    alpha: T, a: &'a [T], lda: usize, b: &'a [T], ldb: usize, beta: T, c: &'a mut [T], ldc: usize }
pub struct Registry { /* providers(), get(id), available(), install(Box<dyn Provider>) */ }
```

Decisions and their reasons:

1. **Typed methods per dtype, not `dyn Any`/byte buffers.** Compile-checked; Fuel's byte storage casts once at its edge
   (it already does, via `as_slice`). Default bodies return `Error::Unsupported`, so a provider lists only what it has.
2. **Explicit leading dimensions** in `GemmArgs`. aocl-blas derives them, oneMKL takes them via `MatrixRef`; the neutral
   form is the superset and both adapt cheaply. Row-major only at the boundary (Fuel's layout); providers translate.
3. **Capabilities are data, not trait bounds**, so the matrix of (provider x op x dtype) can be printed and diffed, and
   tests are generated from it ("every provider that claims gemm_f32 passes the same known-answer and random-vs-reference suite").
4. **No Fuel types in `cpufish`.** Fuel keeps its own adapter (`OpParams::Matmul` to `GemmArgs`); that keeps `cpufish`
   publishable and reusable, and avoids a cycle with Fuel.
5. **Vendor crates are optional dependencies** of per-provider crates (`cpufish-aocl` etc.), each gated by a cargo feature
   on the umbrella `cpufish`. Dependencies are crates.io versions only.
6. **Threading and ISA knobs** (oneMKL `pin_isa`/`ThreadCountGuard`, BLIS `BLIS_NUM_THREADS`) are provider-specific
   config passed at `install` time, surfaced in `capabilities().notes`; no global state changes from `probe`. Added after Fuel review: `fn effective_config(&self) -> Config` reports the thread count and ISA actually in force.
7. **Determinism metadata** (Fuel's `PrecisionGuarantee`: bit-stable on fixed hardware, ulp bounds) is exposed per
   capability later; first cut reports only "claimed", measured by the shared test suite.

## Staging

| Step | Delivers | Checked by |
|---|---|---|
| S1 | `cpufish` core: `Provider`, `Registry`, `portable` provider (pure Rust, `gemm` crate), shared gemm test suite | CI on 3 OSes |
| S2 | `cpufish-aocl` over `aocl-blas` | self-hosted runner (AOCL installed on the box) |
| S3 | `cpufish-onemkl` over crates.io `onemkl` | self-hosted runner (oneMKL 2026.1 installed) |
| S4 | benchmark harness: same gemm shapes across all available providers, one table | local + runner |
| S5 | OpenBLAS provider; then LAPACK/FFT/vector-math ops | per provider |

## Answers from the Fuel lane (2026-10-30, `origin/main@3534c2a6`, message from the Fuel lane)

1. **Fuel uses its own adapter, not 1:1 registration.** A Fuel registration is an FKC contract with accuracy/determinism claims that Fuel verifies in its own ledger before preferring a kernel. `capabilities()` stays printable data (useful to generate contracts and tests); Fuel adds the claims.
2. **Batched gemm is not in the first cut.** Add `Op::GemmBatched` later, driven by a measured shape table from the S4 harness.
3. **conv2d-via-gemm stays in Fuel** (`fuel-conv`); a vendor-native conv would be a separate `Op::Conv`.
4. **Request, adopted in decision 6:** providers report the *effective* thread count and ISA (`effective_config()` on `Provider`, also summarised in `capabilities().notes`), because ULP/bit-stability depends on them.
