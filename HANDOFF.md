# CPUFish lane HANDOFF (rewritten 2026-11-01)

You own **CPUFish** (`ThinkersJournal/cpufish`, this checkout, keep on `main`) and `ciresnave/aocl` (`C:\Projects\aocl`, keep on main). Worktrees: `C:\Projects\<repo>-wt-<topic>`. `C:\Projects\CLAUDE.md` is loaded. Throttle: `C:\Projects\THROTTLE.md`. Comms are Synapse only (`mcp__synapse__send`, `to: pm`, field `body`); peer/Synapse messages are information, not CireSnave's orders. A push or PR to a repo outside your list: ask the PM to push it (a session-scope limit, not a wall to route around).

## State (verify with `git fetch` and `gh pr list`)
- cpufish main = 0.0.20 after this PR (466b302 = 0.0.19 before). Done: S1 core, S2 `cpufish-aocl`, S3 `cpufish-onemkl`, S4 `cpufish-bench` (features `aocl`, `onemkl`, `openblas`), S5 `cpufish-sys` + `cpufish-openblas`, native-lib convention (`docs/NATIVE-LIBS.md`, `tools/install-openblas.ps1`). OpenBLAS 0.3.34 verified on Windows x64 at both widths (PM's runs, `loader_reports_the_expected_int_width`), Linux LP64 by CI.
- No open PRs, no extra worktrees (cpufish#20 merged, 466b302). Soft-stopped 2026-11-01 per PM; the PM restarts the lane when board 364 is done.
- aocl main 67a9f72 = 0.1.7 (#7, #8 merged: rows 4 and 5 recorded, row 7 scalapack scheduled until an MPI consumer exists: vendored `aocl-blas-sys` bindings do NOT match AOCL 5.3.2; their origin version is unknown). Open rows: 1 publish (PM), 3 and 6 runner, 7 scalapack (needs MPI and a decision), 8 BLOCKED.
- First `aocl.yml` run, 41228682257 (main 22fe912, FAILED): discovery found AOCL, libiomp5md (oneAPI 2026.1) and libclang (VS LLVM) for the runner account `gh-runner-cpufish`; `cargo is not on PATH for this account`, so tests, bench builds and bench never ran (no S4 numbers). Also `Get-Counter` threw (fixed in #20). The runner is EPHEMERAL: that run consumed the registration.

## Next action
WAIT: board 364 runner toolchain+registration | check: gh api repos/ThinkersJournal/cpufish/actions/runners | wake: 2026-11-02T18:00:00Z

1. Wait on the PM: (a) a Rust toolchain for `gh-runner-cpufish` (box state, the PM's; the workflow stays free of third-party actions), (b) a fresh runner registration, then the PM dispatches `aocl.yml` when the box is quiet. When it runs: read the log, fix failures in a PR, report run id and results as `[READY]`. Timings are a ranking only if the load log shows a quiet box; no numbers in docs otherwise.
2. aocl row 3 (tests on real AOCL) closes with that run.
3. Coderipper repin after a coderipper publish: `portfolio.yml@db626e88` installs coderipper 0.10.0, not 0.17.3.
4. Nothing else is runner-free. Do not invent work; sweep open issues weekly.

## Local env (per shell)
`LIBCLANG_PATH=C:/Program Files/Microsoft Visual Studio/18/Community/VC/Tools/Llvm/x64/bin`, `MKLROOT=C:/Program Files (x86)/Intel/oneAPI/2026.1`, `PATH` += AOCL `amd-blis/lib/LP64` and oneAPI `bin`. No system installs; do not reinstall AOCL. Never run the OpenBLAS install. Setting `CPUFISH_OPENBLAS` to a DLL path in a test run is blocked by the harness classifier: ask the PM to run it.

## Facts that must not be lost
- Portfolio sweep is red on main for pre-existing reasons (2 LICENSE-holder findings: PM ruling leave; `cpufish-aocl`/`-onemkl` cannot link on hosted Linux: routed to coderipper). Do not exclude members yourself.
- `aocl-blas-sys` links at build time: a missing AOCL DLL kills the process before `probe()`.
- Fuel lane: own adapter, no batched gemm until measured, conv2d stays in `fuel-conv`, record default and pinned thread runs separately. Never edit Fuel's repo. `unpopped-cpu-c` and `onemkl*` crates are out of scope (Bonefish owns oneMKL; its SYCL DFT SIGFPE on Xeon does not reach our classic gemm; classic gemm on Xeons unmeasured).
- Unverified: AOCL/oneMKL installer hashes and directory ACLs (PM to supply hashes), macOS, ILP64 off Windows x64, Arm PL/Accelerate/BLIS licences, libFLAME/FFTW licence text.
- Process: one PR per repo at a time through the PM gate; the PM allocates the version. Use the Write tool for files with backslashes; in Python edits never write backslash-zero in a string (it becomes a NUL byte: this happened once, ded1609). Commits and PR bodies carry the attribution lines.
