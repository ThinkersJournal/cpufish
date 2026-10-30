# Measurements

## S2 `cpufish-aocl`, 2026-10-30, local Windows box (not CI)

Run: `cargo test -p cpufish-aocl -- --nocapture` at the S2 PR head, `aocl-blas`/`aocl-blas-sys`/`aocl-types` 0.1.0 from crates.io.
Environment: AOCL 5.3.2 at `C:\Program Files\AMD\AOCL-Windows` (LP64 `AOCL-LibBlis-Win-MT-dll`); `PATH` also needed
`...\amd-blis\lib\LP64` and `libiomp5md.dll` (from oneAPI 2026.1 `bin`), else the process dies at load with
`STATUS_DLL_NOT_FOUND` (the link-time-provider cost in the design note, measured).

```
aocl version=AOCL-BLAS 5.3.0 Build 20260511 config=Config { threads: Some(32), isa: Some("zen4") }
test aocl_reports_version_threads_and_isa ... ok
test aocl_probe_succeeds ... ok
test aocl_passes_conformance ... ok
test result: ok. 3 passed; 0 failed
```

Finding: the first conformance run failed at `k=0` (`aocl-blas` demands an A slice longer than the Provider contract
promises). The adapter now does `C := beta*C` itself for `k == 0`. Ideal home for the fix is upstream in `aocl-blas`
(aocl ROADMAP row 8 candidate).

Also, `aocl` 0.1.5 (aocl#6, unpublished) `cargo test -p aocl-blas` on the same install: 53 passed, with bindgen 0.73.

### Provenance of the local libclang (bindgen needs it; the box has none)

PM ruling 2026-10-30: pip wheel in a scratch venv outside any repo, local runs only. `pip` package `libclang` 18.1.1,
wheel `libclang-18.1.1-py2.py3-none-win_amd64.whl`, sha256 `4dd2d3b82fab35e2bf9ca717d7b63ac990a3519c7e312f19fa8e86dcc712f7fb`
(computed from the downloaded file), venv `C:\Projects\.claude-hooks\libclang-venv`, `LIBCLANG_PATH` set per shell.

## S3 `cpufish-onemkl`, 2026-10-30, local Windows box (not CI)

Run: `cargo test -p cpufish-onemkl -- --nocapture` at the S3 PR head, `onemkl`/`onemkl-sys` 0.2.0 from crates.io.
Environment: oneMKL 2026.1 at `C:\Program Files (x86)\Intel\oneAPI6.1`; `MKLROOT` set for the build, `...6.1in` on `PATH` for
`mkl_rt.3.dll` (else `STATUS_DLL_NOT_FOUND` at load). Note: from Git Bash the `PATH` entry must be POSIX style (`/c/Program Files (x86)/...`).

```
onemkl version=Intel(R) oneAPI Math Kernel Library Version 2026.1-Product Build 20260612 for Intel(R) 64 architecture applications config=Config { threads: Some(1), isa: Some("Intel(R) Architecture processors") }
test result: ok. 3 passed; 0 failed        (default: threading-sequential)
```

With `--no-default-features --features threading-intel-openmp`: 3 passed, `threads: Some(16)`.
`isa` is oneMKL's processor-family string from `MKL_Get_Version`; it does NOT name the vector-ISA tier in force (no query for that
in `onemkl` 0.2), so consumers should not read it as one.

## S4 bench harness, 2026-10-30, local Windows box (not CI)

Run: `cargo run --release -p cpufish-bench --features aocl,onemkl` at the S4 PR head (median of 5 runs after 1 warm-up).
Same environment as S2/S3 above. oneMKL used the default **sequential** layer, AOCL reported 32 threads, portable is single-threaded.

## Providers

- `portable` version=None config=Config { threads: Some(1), isa: Some("runtime-detected by the gemm crate") }
- `aocl` version=Some("AOCL-BLAS 5.3.0 Build 20260511") config=Config { threads: Some(32), isa: Some("zen4") }
- `onemkl` version=Some("Intel(R) oneAPI Math Kernel Library Version 2026.1-Product Build 20260612 for Intel(R) 64 architecture applications") config=Config { threads: Some(1), isa: Some("Intel(R) Architecture processors") }

f32, NoTrans/NoTrans, alpha=1, beta=0, median of 5 runs after 1 warm-up. GFLOP/s = 2mnk / time.

(Timing table withheld from this document: PM ruling 2026-10-30 says no number goes in a doc before the Fuel lane confirms the real shapes. Fuel replied it has no fixed list, so the shapes stay a provisional superset. The harness prints the table.)

**Timings from this harness are NOT a ranking of providers: the box is shared and loaded, and thread use is unconfirmed.** The box is shared with about 20 other agent lanes running builds, so the CPU was not
idle (not measured; the load was not sampled). All three providers land near 60 GFLOP/s on large squares, which is
more likely contention or an unused thread pool than parity: AOCL's reported 32 threads did not show up as speedup, and no
check was made that BLIS actually ran multi-threaded. Treat this as proof the harness runs and the providers agree on
the answer (max abs error vs portable <= 1.8e-4 on values in [-1, 1]), not as a ranking. Re-measure on a quiet box or the
self-hosted runner before any provider choice.
