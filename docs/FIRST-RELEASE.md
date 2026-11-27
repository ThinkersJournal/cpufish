# First-release readiness checklist (a check, not a publish)

Asked by pm-kernels 2026-11-27 ([TASK] first-release-readiness; PM: GO for the check only). **cpufish stays unpublished and
unqueued.** Facts only; anything not read or run is marked UNVERIFIED. Measured at `origin/main` 4a4f42c (0.0.30), on
rustc 1.100.0 / cargo 1.100.0, Windows 11, 2026-11-27. **Updated on the `first-release-readiness` branch after the PM rulings of 2026-11-27: items marked FIXED below are fixed on that branch (not on `main` until its PR merges).** The licence and provenance *decision* is CireSnave's.

Verdict per item: **PASS** / **GAP** (a fact that blocks or needs a decision) / **UNVERIFIED**.

## 1. README hello-world and the fresh-eyes trial

- **FIXED: the README has an example** (`examples/hello.rs`, copied into the README with its output
  `portable computed [19.0, 22.0, 43.0, 50.0]`). `cargo test` builds it; the CI `test` job then runs
  `cargo run --example hello` and `grep -F`s that output, so the README text and the example cannot drift apart silently
  (the README copy itself is not machine-compared to the file: UNVERIFIED against drift in the README text only).
  Run by hand on this branch: prints the documented line.
- Ease observation (facts, not a ruling): a 2x2 multiply takes a 13-field `GemmArgs` with explicit leading dimensions
  and transposes; there is no `provider.gemm(a, b)` convenience. The first example can only be `portable`, since a vendor
  provider needs its library installed.
- **6.24 applies to cpufish** (PM ruling via pm-kernels, 2026-11-27: developer-facing, though not an Ant). **Fresh-eyes
  trial: NOT yet run.** It goes after this PR merges (a Sonnet sub-agent, model pinned, given only the README, must reach
  the example's output) and before the first non-placeholder release; its summary goes in the PR body or a follow-up.

## 2. Licences, NOTICE, and what is linked vs shipped

- **Our files:** `LICENSE-MIT` and `LICENSE-APACHE` exist at the repo root; every crate declares `MIT OR Apache-2.0`.
  **No `NOTICE` file** exists (`git ls-files` lists none). **FIXED on the branch:** every subcrate now carries copies of `LICENSE-MIT` and `LICENSE-APACHE` (as measured at 4a4f42c, only the root crate did). Only
  `cpufish` and `cpufish-sys` are the intended first publish (the others are `publish = false` and see section 7).
- **Does any crate ship library bytes? No.** Package contents (`cargo package --list`): `cpufish` and every subcrate are
  `Cargo.toml`, `src/`, `tests/`, and for the root crate also the docs, `tools/install-openblas.ps1`, CI and
  `HANDOFF.md`/`ROADMAP.md` (no `exclude` is set; whether to ship those is a call). No `.dll/.so/.lib/.a/.dylib` is in
  any list. The dependency crates the providers use also ship none: `aocl-blas-sys 0.2.5` (12 files),
  `aocl-build 0.2.5` (10), `onemkl-sys 0.6.0` (9; file count measured at 0.5.0, not re-counted), read from the registry sources with the extension filter above (0 binary files;
  control: the same filter lists `.rs`, `.toml` files, so it sees files).
- **Per dependency, linked vs shipped, and conditions read** (sources and quotes are in `docs/NATIVE-LIBS.md` "Hosted CI",
  read 2026-11-01; not re-read today):

  | Library | How it reaches a user | Shipped in a cpufish crate? | Licence read |
  |---|---|---|---|
  | OpenBLAS | loaded at run time by `cpufish-sys` from the user's own install (`libloading`) | No | BSD-3 (`OpenMathLib/OpenBLAS` LICENSE) |
  | AOCL-BLAS (BLIS) | linked at build time by `aocl-blas-sys` (`links = "amd-blis"`) against the user's install | No | BSD-3 plus MIT (`amd/blis` LICENSE, v5.3.2). **AMD's own EULA/download pages: NOT read** (amd.com timed out) |
  | oneMKL | linked at build time by `onemkl-sys` (`links = "mkl"`) against the user's install | No | Intel Simplified Software License (Oct 2022): binary-only redistribution without modification, no reverse engineering; Intel's FAQ: "redistribution is allowed per the terms of the ISSL". The ISSL page was read to 100000 of 149214 characters, so **UNVERIFIED** past that |
  | `gemm` 0.19 (used by `portable`) | Rust dependency | source, via cargo | MIT |
  | `libloading` 0.9 | Rust dependency | source, via cargo | ISC |

- **GPL conditions:** none found. `cargo metadata --locked` over all 98 locked packages lists 0 whose licence expression
  contains `GPL` (control: the same query lists `gemm` MIT, `libloading` ISC and the `aocl`/`onemkl` crates, so it reads the
  field). Not checked: the licences of the libraries' own build-time dependencies on the **user's** machine
  (AOCL's other components such as libFLAME/FFTW, which cpufish does not link: UNVERIFIED, previously listed in HANDOFF).
- **Redistribution conditions that would apply if a binary containing a vendor library were ever shipped:** BSD-3 (OpenBLAS,
  BLIS) requires the notice and licence text to travel with the binary; ISSL requires the ISSL terms to travel and
  forbids modification. cpufish ships none, so these are not triggered today.
- **Cargo.deny:** `deny.toml` allows MIT, Apache-2.0 (+LLVM-exception), BSD-2/3, ISC, Unicode-3.0, Zlib, CC0-1.0;
  `[sources]` denies unknown registries and git; one advisory is ignored (RUSTSEC-2024-0436 `paste`, via `gemm`, PM
  ruling 2026-10-30).

## 3. Crate metadata

All six crates: `version = 0.0.30`, `edition = 2024`, `rust-version = 1.95`, `license = "MIT OR Apache-2.0"`,
`repository = https://github.com/ThinkersJournal/cpufish`, `publish = false`, a `description`. **FIXED on the branch:**
each now also has `readme = "README.md"` (a README file exists in every crate dir: `cpufish-sys`, `-aocl`, `-onemkl`,
`-openblas` got a short one), `keywords = ["blas", "gemm", "linear-algebra", "cpu"]` and
`categories = ["mathematics", "science"]`. Still not set: `homepage`, `documentation` (crates.io/docs.rs default to the
repository and docs.rs; a call, not a gap). `publish = false` stays until the PM lifts it per crate; path dependencies
already carry a `version`.

## 4. MSRV stated and tested

- **Stated:** `rust-version = "1.95"` in all six crates.
- **FIXED: tested in CI.** New job `msrv (1.95)` runs `cargo check --locked --all-targets` on exactly 1.95 (not yet a
  required check on `main`).
- **Measured by hand 2026-11-27 (Windows):** `cargo +1.95 check --locked --all-targets` passes for the default members.
  `cargo +1.94.0` refuses ("requires rustc 1.95"), as cargo enforces the field. With `--ignore-rust-version`,
  `cargo +<v> check --locked` of the default members finished on 1.94.0, 1.90, 1.88 and 1.85; `-p cpufish-openblas`,
  `-bench`, `-onemkl` and `-aocl` finished on 1.85 too, and `--all-targets` of the default members on 1.85. So the
  code itself builds back to the edition-2024 floor (1.85); what sets 1.95 is the workspace's own declaration: `onemkl`
  and `onemkl-sys` (ours, on crates.io) declare `rust-version = 1.95`. The number is kept at 1.95 for all crates (one
  version rule) and the CI job holds it; lowering the core crates alone is a call for PM/CireSnave. Control: 1.94 is
  refused without the flag, so the flag was doing its work.

## 5. No path or git dependencies

- `git grep` for a dependency table line with `path =` or `git =` in `*.toml`: 8 hits, **all** workspace-internal
  `cpufish*` crates, each with a `version =` (list in the PR body). Zero hits name another repo or an external path.
  Control: the same pattern finds those 8, so it matches the syntax in use.
- `Cargo.lock`: 92 `source = "registry+https://github.com/rust-lang/crates.io-index"` entries, 0 `git+` sources, 98 `name =`
  entries (the 6 workspace crates are the 6 without a `source`). Every external dependency is a crates.io version, including
  `aocl-*` 0.2.5 and `onemkl` 0.6.1 (the `ciresnave`/ThinkersJournal crates are on crates.io, per CLAUDE.md section 9).
- **Latest-version check (crates.io API, 2026-11-27; every direct dependency of the six crates, one row per dependency):**

  | Crate | Dependency | Requirement | Locked (was) | Latest | Note |
  |---|---|---|---|---|---|
  | cpufish | gemm | ^0.19 | 0.19.0 | 0.19.0 | current |
  | cpufish, cpufish-sys | thiserror | ^2 | 2.0.21 | 2.0.21 | current (1.0.69 also locked, transitive via `gemm`) |
  | cpufish-sys | libloading | ^0.9 | 0.9.1 (was 0.9.0) | 0.9.1 | lock moved on this branch; 0.8.9 also locked, transitive |
  | cpufish-aocl | aocl-blas, aocl-blas-sys, aocl-types | ^0.2 | 0.2.5 | 0.2.5 | current |
  | cpufish-onemkl | onemkl | ^0.6 | 0.6.1 | 0.6.1 | current |
  | cpufish-bench | clap | ^4.6.8 | 4.6.10 (was 4.6.8) | 4.6.10 | lock moved on this branch |

  No lag remains among direct dependencies. Not checked: the latest of transitive dependencies other than those two `cargo
  update` steps touched (the lock has 7 and 6 "unchanged dependencies behind latest" notes from cargo; their reasons are not
  investigated here, UNVERIFIED).

## 6. Yank policy

- **No cpufish-specific yank policy exists in this repo** (`grep` of `CHANGELOG.md` and `ROADMAP.md` for "yank": 0 hits;
  control: the same search finds "placeholder" and "publish").
- Portfolio facts read: CireSnave 2026-10-11 (`CIRESNAVE-EXPECTATIONS.md` section 6.11, verbatim there): *"We can always
  yank anything that is wrong after the fact."* — the PM may publish any release without asking; yanks have been used
  elsewhere (auth-framework 0.4.2, item 32). **Yank policy (written on this branch, now also in `CHANGELOG.md`):** a version is yanked only for
  a correctness bug that silently gives wrong numbers, a security problem, or a licence/provenance error; an ordinary bug is
  fixed forward in a patch release; each yank is recorded in `CHANGELOG.md` with the reason and the fixing version.

## 7. Crate names

**FIXED (PM, 2026-11-27, via pm-kernels):** all six workspace names are reserved by us on crates.io (the PM published 0.0.0
placeholders for `cpufish-aocl`, `cpufish-onemkl`, `cpufish-openblas`, `cpufish-bench`; `cpufish` and `cpufish-sys` were
reserved 2026-10-30). Read on 2026-11-27 before that ruling: both placeholders existed, the four others returned 404;
UNVERIFIED by me after the PM's publish (not re-read). The placeholders' repository field was empty and can only change
with a release.

## Summary of what remains for the PM

Fixed on the branch: README example with a CI run; MSRV job; licence files in each subcrate; keywords/categories/readme;
yank policy; latest-version check (two lock moves); names (PM). **Remain:** the 6.24 fresh-eyes trial (after merge, before
the first non-placeholder release); no `NOTICE` file; AMD's EULA not read; ISSL read only to 100000 of 149214 characters;
whether to ship `HANDOFF.md`/`ROADMAP.md`/CI files in the root package (`exclude`); lowering the core crates' MSRV below
1.95 (builds to 1.85); the first-release decision itself (PM).
