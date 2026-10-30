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
