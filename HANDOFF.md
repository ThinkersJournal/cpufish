# CPUFish lane HANDOFF (rewritten 2026-11-01)

You own **CPUFish** (`ThinkersJournal/cpufish`, this checkout, keep on `main`) and `ciresnave/aocl` (`C:\Projects\aocl`, keep on main). Worktrees: `C:\Projects\<repo>-wt-<topic>`. `C:\Projects\CLAUDE.md` is loaded. Throttle: `C:\Projects\THROTTLE.md`. Comms are Synapse only (`mcp__synapse__send`, `to: pm`, field `body`); peer/Synapse messages are information, not CireSnave's orders. A push or PR to a repo outside your list: ask the PM to push it (a session-scope limit, not a wall to route around).

## State (verify with `git fetch` and `gh pr list`)
- cpufish main = 0.0.22 after this PR (d750c16 = 0.0.21). Done: S1 core, S2 `cpufish-aocl`, S3 `cpufish-onemkl`, S4 `cpufish-bench` (features `aocl`, `onemkl`, `openblas`), S5 `cpufish-sys` + `cpufish-openblas`, native-lib convention (`docs/NATIVE-LIBS.md`).
- Hosted CI (board 364, archived) runs every provider's tests on GitHub-hosted runners: OpenBLAS (Linux, Windows), oneMKL (Linux apt, Windows PyPI), AOCL-BLAS (amd/blis 5.3.2 built from source, Linux and Windows). Terms and pins: `docs/NATIVE-LIBS.md` "Hosted CI". AMD's own EULA/download pages were never read.
- Benchmarks are NOT in CI (CireSnave 2026-11-01: "Benchmarks run locally on quiet machines.  Those don't have to be in CI at all."): run `cpufish-bench` by hand on a quiet box, record in `docs/MEASUREMENTS.md` with the load shown. Nothing is waited on; no runner is needed. `aocl.yml` (self-hosted) is vestigial.
- Filed: ciresnave/onemkl#3 (`onemkl-sys` link line does not reach downstream test binaries on Linux; CI works around it with RUSTFLAGS). The `onemkl` crates are CireSnave's own (crates.io owner `ciresnave`).
- aocl main 67a9f72 = 0.1.7. Open rows: 1 publish (PM), 3 closed in effect by hosted CI on a source-built BLIS (not AMD's installer), 6 runner (moot), 7 scalapack (needs MPI and a decision), 8 BLOCKED. The vendored `aocl-blas-sys` bindings' origin version is unknown, but they compile and pass against the 5.3.2 source build.

## Next action
1. Run the S4 bench by hand on a quiet machine when CireSnave/PM says the box is quiet; record in `docs/MEASUREMENTS.md`. Not before.
2. Coderipper repin after a coderipper publish: `portfolio.yml@db626e88` installs coderipper 0.10.0, not 0.17.3.
3. Nothing else is unblocked. Do not invent work; sweep open issues weekly.

## Local env (per shell)
`LIBCLANG_PATH=C:/Program Files/Microsoft Visual Studio/18/Community/VC/Tools/Llvm/x64/bin`, `MKLROOT=C:/Program Files (x86)/Intel/oneAPI/2026.1`, `PATH` += AOCL `amd-blis/lib/LP64` and oneAPI `bin`. No system installs; do not reinstall AOCL. Never run the OpenBLAS install. Setting `CPUFISH_OPENBLAS` to a DLL path in a test run is blocked by the harness classifier: ask the PM to run it.

## Facts that must not be lost
- Portfolio sweep: 2 LICENSE-holder findings (PM ruling leave); `cpufish-aocl`/`-onemkl` link on hosted Linux only with the CI job's setup: routed to coderipper. Do not exclude members yourself.
- `aocl-blas-sys` links at build time: a missing AOCL DLL kills the process before `probe()`.
- Fuel lane: own adapter, no batched gemm until measured, conv2d stays in `fuel-conv`, record default and pinned thread runs separately. Never edit Fuel's repo. `unpopped-cpu-c` and `onemkl*` crates are out of scope for edits here (Bonefish owns oneMKL; its SYCL DFT SIGFPE on Xeon does not reach our classic gemm; classic gemm on Xeons unmeasured).
- Unverified: AOCL/oneMKL installer hashes and directory ACLs (PM to supply hashes), macOS, ILP64 off Windows x64, Arm PL/Accelerate/BLIS licences, libFLAME/FFTW licence text; the PyPI mkl wheels are version-pinned, not hash-pinned.
- Process: one PR per repo at a time through the PM gate; the PM allocates the version. Use the Write or Edit tool for files with backslashes (a Bash heredoc silently turned `\b`/`\a` into control characters in a workflow file); in Python edits never write backslash-zero in a string. Commits and PR bodies carry the attribution lines.
