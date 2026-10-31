# CPUFish lane HANDOFF (rewritten 2026-10-30)

You own **CPUFish**: the CPU-backend crate set for Fuel plus the CPU math-library bindings. Repo `ThinkersJournal/cpufish` (this checkout, keep on `main`); crates `cpufish`, `cpufish-sys` reserved at 0.0.0. You also own `ciresnave/aocl` (checkout `C:\Projects\aocl`, keep on main; worktrees `C:\Projects\<repo>-wt-<topic>`). Read `C:\Projects\CLAUDE.md` (loaded). Onboarding already read. Throttle: `C:\Projects\THROTTLE.md` (FULL SPEED at 2026-10-30).

## State (verify with `git fetch` and `gh pr list`)
- cpufish main 653ac37 = 0.0.7: S1 core, S2 `cpufish-aocl`, S3 `cpufish-onemkl`, S4 `crates/cpufish-bench` all merged; no open PRs, no extra worktrees. aocl main 5d361cb = 0.1.5 (deps current), no open PRs.
- AOCL/oneMKL tests pass LOCALLY only, UNVERIFIED in CI (no self-hosted runner; rig dual-boots, runners planned). Timings are NOT a ranking (loaded shared box): no numbers in docs until re-measured.
- CireSnave ruling (board 329): Bonefish owns oneMKL and the onemkl crates; we consume from crates.io. oneMKL on AMD proves the provider, says nothing of Intel hardware.
- Fuel lane: no fixed gemm list (shapes = model matmuls, one gemm per slot; decode m=1, prefill m=T); AOCL/MKL run at library-default threads.
- aocl ROADMAP open rows: 1 publish gap (PM), 3 tests on real AOCL (runner), 5 bindings provenance (blas-sys built against real headers with bindgen 0.73 OK), 6 Linux, 7 scalapack, 8 API gaps (candidate: `aocl-blas` k=0 shape check wants longer A/B than needed; the adapter works around it).

## Next action (BEGIN here)
PM offered, my order: ROADMAP row 8 (a) explicit threads setting: harness `--threads N` plus provider support (`with_threads`-style; BLIS/MKL thread setting is process-global, so design first: apply once at startup, report effective config after), record library defaults separately; then (b) S5 OpenBLAS / loader `cpufish-sys` design. Do (a) first; one PR, version 0.0.8; update ROADMAP row 8; this HANDOFF.md edit is uncommitted: include it in that PR.
Local AOCL/oneMKL env per shell: `LIBCLANG_PATH=C:/Projects/.claude-hooks/libclang-venv/Lib/site-packages/clang/native` (PM-approved pip libclang 18.1.1, local only), `MKLROOT=C:/Program Files (x86)/Intel/oneAPI/2026.1`, and `PATH` += `/c/Program Files/AMD/AOCL-Windows/amd-blis/lib/LP64` and `/c/Program Files (x86)/Intel/oneAPI/2026.1/bin` (POSIX style in Git Bash). No system installs without CireSnave. Builds of the onemkl crates take ~5 min; run clippy in background.

## Holds and facts that must not be lost
- `aocl-blas-sys` links at build time: a missing AOCL DLL kills the process before `probe()`; link-time provider now (option A, Fuel lane OK'd 2026-10-30), dlopen loader in `cpufish-sys` later (option B).
- Self-hosted runner: not available (install is agentlife's held RUNNER-SETUP row; PM raising it). Policy 5.1g/5.1h applies once it exists.
- Fuel lane answers (2026-10-30): own adapter, not 1:1 registration; no batched gemm until measured; conv2d-via-gemm stays in `fuel-conv`; report effective threads/ISA.
- UNVERIFIED in SURVEY: Arm PL / Accelerate / upstream BLIS licences; libFLAME and FFTW licence text.
- Out of scope: `unpopped-cpu-c` (Unpopped's), `onemkl*` crates (Bonefish lane; consume only). Never edit Fuel's repo.
- Rules that bit this session: shell heredocs with embedded quotes can fail to parse (use the Write tool for source files); `cargo update -w --offline` regenerates the lockfile version after a bump; write PR bodies/commits with the attribution lines from the system reminder.
- Peers: PM = cwd `C:\Projects` (no repo), Fuel lane = cwd `C:\Projects\fuel`; IDs rotate on restart, use `list_peers`.
