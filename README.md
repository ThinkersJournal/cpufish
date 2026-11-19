# cpufish

A vendor-neutral CPU math layer for Fuel, with per-vendor providers selected at run time. Pre-1.0 (0.0.x): the API is not stable. Only gemm is covered so far.

## Crates

| Crate | What it is |
|-------|------------|
| `cpufish` | Core: the `Provider` trait, `Registry`, an always-available `portable` provider, and a conformance suite every provider must pass (`cpufish::conformance`) |
| `cpufish-aocl` | AMD AOCL (BLIS) provider, link-time, over the `aocl-blas` crate |
| `cpufish-onemkl` | Intel oneAPI MKL provider, link-time, over the `onemkl` crate |
| `cpufish-openblas` | OpenBLAS provider, loaded at run time through `cpufish-sys` |
| `cpufish-sys` | Run-time loader for CBLAS-ABI libraries (OpenBLAS first) |
| `cpufish-bench` | Benchmark harness comparing the installed providers on gemm shapes; see its README |

`cpufish-aocl`, `cpufish-onemkl` and `cpufish-openblas` need the vendor library installed, so they are workspace members but not default members. Where each library lives and how CPUFish finds it: `docs/NATIVE-LIBS.md`. Design: `docs/PROVIDER-DESIGN.md`. Status and open rows: `ROADMAP.md`.

## Check it yourself

```
cargo test -p cpufish          # core + portable provider, no vendor library needed
cargo test -p cpufish-sys      # loader
cargo test -p cpufish-aocl     # needs AOCL
cargo test -p cpufish-onemkl   # needs oneMKL
cargo test -p cpufish-openblas # needs OpenBLAS
```

CI runs every provider's tests against the real library on GitHub-hosted runners (Linux and Windows); terms and pins are in `docs/NATIVE-LIBS.md` "Hosted CI". Benchmarks are not in CI: timings need a quiet machine and are run by hand.

The crate names `cpufish` and `cpufish-sys` are reserved on crates.io at 0.0.0; the code here is not yet published.

Licensed under MIT or Apache-2.0, at your option.
