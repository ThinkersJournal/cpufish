# cpufish-bench

Compares the installed CPUFish providers on gemm shapes and checks each answer against `portable`.

```
cargo run --release -p cpufish-bench --features aocl,onemkl -- [--quick] [--reps N]
```

**Timings from this harness are NOT a ranking of providers: the box is shared and loaded, and thread use is unconfirmed.** Re-measure on a quiet machine before choosing a provider.

- Shapes are a superset of what Fuel calls, not Fuel's measured histogram: the Fuel lane (2026-10-30) has no fixed gemm
  list; shapes come from the model's matmuls, one gemm per batch slot (decode is m=1, prefill m=T).
- Threads: Fuel's AOCL and MKL backends set no thread count and run at library defaults. The harness does not set one
  either; it prints the default each provider reports. To pin, set `BLIS_NUM_THREADS` / `MKL_NUM_THREADS` in the
  environment and record it with the result. An explicit `--threads` setting is future work (setting threads is
  process-global).
- oneMKL on an AMD CPU is a valid test of the provider, but says nothing about Intel hardware.
