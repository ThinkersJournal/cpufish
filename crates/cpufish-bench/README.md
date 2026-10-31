# cpufish-bench

Compares the installed CPUFish providers on gemm shapes and checks each answer against `portable`.

```
cargo run --release -p cpufish-bench --features aocl,onemkl -- [--quick] [--reps N] [--threads N]
```

**Timings from this harness are NOT a ranking of providers: the box is shared and loaded, and thread use is unconfirmed.** Re-measure on a quiet machine before choosing a provider.

- Shapes are a superset of what Fuel calls, not Fuel's measured histogram: the Fuel lane (2026-10-30) has no fixed gemm
  list; shapes come from the model's matmuls, one gemm per batch slot (decode is m=1, prefill m=T).
- Threads: Fuel's AOCL and MKL backends set no thread count and run at library defaults. By default the harness sets
  none either and prints the default each provider reports. `--threads N` calls `Provider::set_threads(N)` once, before
  any gemm (the setting is process-global in BLIS and MKL), and the report shows the effective config afterwards, so
  check it took. Record library-default runs and pinned runs separately.
  The bench's oneMKL is the sequential layer by default, which ignores the request (effective threads stay 1); use the `onemkl-openmp` feature to make it take effect. `portable` is single-threaded and refuses anything but 1.
- oneMKL on an AMD CPU is a valid test of the provider, but says nothing about Intel hardware.
