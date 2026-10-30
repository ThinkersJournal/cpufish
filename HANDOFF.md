# CPUFish lane HANDOFF (new lane, created 2026-10-30 by the PM)

You own **CPUFish**: the CPU-backend crate set for Fuel, and the CPU math-library bindings that go with it. Crates reserved on crates.io at 0.0.0: `cpufish`, `cpufish-sys` (owner ciresnave). Repo: ThinkersJournal/cpufish (this checkout, `C:\Projects\cpufish`). You also take over the existing `ciresnave/aocl` repo (checkout `C:\Projects\aocl`, wrapper crates such as `aocl-blas` and `aocl-types` 0.1.0, which Fuel depends on from crates.io).

CireSnave, 2026-10-30, verbatim: *"I think we need a separate lane for each Fuel backend: Baracuda, Vulkane, Bonefish, Rockfish, and possibly one for the CPU backend although it includes AOCL which, in Intel's software, has its equivalent as part of OneAPI."* and *"Go with CPUFish for the CPU crate."* And on goals: *"so we can really test the limits of every manufacturer's software stack."*

## Facts the PM measured
Fuel's manifest depends on `aocl-blas = "0.1"` and `aocl-types = "0.1"` from crates.io (switched from a path dependency on 2026-06-13); `fuel-aocl-cpu-backend` (in the Fuel repo) uses them; building it needs libclang for bindgen and an AMD AOCL install (BLIS) on the machine. The ciresnave/aocl crates are published (aocl-blas 0.1.0, 2026-04-27). Intel's equivalent of AOCL is the CPU interface of oneMKL: our `onemkl`/`onemkl-sys` crates (repo `C:\Projects\onemkl`, owned by the Bonefish lane, PM proposal awaiting CireSnave); consume them from crates.io, do not take them over. Fuel also has its own CPU kernels (see Fuel's ROADMAP and `fuel-cpu-*` crates). **Verify each claim yourself.**

## Scope
- Survey first (docs only): what Fuel's CPU path is today (backends, kernels, AOCL use), what AMD AOCL covers beyond BLAS (LAPACK, FFT, sparse, RNG, libm, etc.), what oneMKL's CPU interface covers, which other CPU math libraries matter (OpenBLAS, BLIS, Apple Accelerate, ARM Performance Libraries, CPU-side SIMD kernels), licences, Windows AND Linux, x86-64 AND aarch64. Options first, recommendation, no code.
- Then: a comprehensive, vendor-neutral CPU layer with per-vendor providers (AOCL, oneMKL, others) selected at run time, so Fuel can test each vendor's CPU stack against the same work. Design with the Fuel lane; do not edit Fuel's repo.
- The `aocl` repo: take over its ROADMAP; keep Fuel's published dependency working (SemVer; coordinate with the Fuel lane before any breaking release).
- Parser/emitter: Unpopped's CPU C emitter (`unpopped-cpu-c`) is Unpopped's today; do not take it unless CireSnave rules.

(see COMMON rules below)
## Rules that bite here
Read `C:\Projects\CLAUDE.md` (loaded) and `C:\Projects\AGENT-ONBOARDING.md` once. Work in a worktree (`C:\Projects\<repo>-wt-<topic>`), never branch in the primary checkout. Version bump on every pushed change (pre-1.0 breaking = the SECOND number); the PM allocates the number at gate time. crates.io versions only, no path or git dependencies; all dependencies on their latest versions. Trace and verify every third-party file against its origin and keep its licence and credit (CireSnave: "I am not a plageurist"). Every claim verified on the forge or by measurement; every absence needs a positive control. Compact `[READY]`/`[STATUS]` messages, short. Peers change ID on every restart: `list_peers`, match on cwd. Restart at task boundaries (HANDOFF rewrite, `lane-restart assert-idle` last, then `--self`). This is an open-source project from day one: no secrets, no private paths in committed files.

## First deliverables (all lanes of this family)
0. `ROADMAP.md` with rows and statuses; a CI workflow (fmt, clippy, tests, cargo-deny, and the portfolio checks workflow the other repos use: copy the shape from `C:\Projects\capant\.github`) in your first PR; then `[ASK]` the PM to enable branch protection requiring those checks (CI and protection everywhere).
1. A survey and design note (options first, recommendation, no code) before any crate code.
