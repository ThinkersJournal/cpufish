# First-release readiness checklist (a check, not a publish)

Asked by pm-kernels 2026-11-27 ([TASK] first-release-readiness; PM: GO for the check only). **cpufish stays unpublished and
unqueued.** Facts only; anything not read or run is marked UNVERIFIED. Measured at `origin/main` 4a4f42c (0.0.30), on
rustc 1.100.0 / cargo 1.100.0, Windows 11, 2026-11-27. The licence and provenance *decision* is CireSnave's.

Verdict per item: **PASS** / **GAP** (a fact that blocks or needs a decision) / **UNVERIFIED**.

## 1. README hello-world and the fresh-eyes trial

- **GAP: the README has no example.** It has a crate table and `cargo test` lines. There is no hello-world, so a stranger
  cannot copy one. `src/lib.rs` has no doc example either: `grep` for a code fence in `src/*.rs` finds none (the only
  tests that use the API are `tests/portable.rs`, which are tests, not an example).
- **Compiled in CI?** Not applicable yet, for the same reason. What CI would compile today: `cargo test --locked
  --all-features` (3 OSes) compiles `tests/` and doc examples (there are none); `cargo doc` runs with `-D warnings`.
  Once an example exists, `examples/hello.rs` plus the existing `cargo test` job would compile it (cargo builds examples
  during `cargo test`) and a README `include_str!`/doctest would tie the README text to it. UNVERIFIED until built.
- **Draft of the intended example** (about 13 lines; compiled and run once by hand in a scratch `examples/hello.rs` at
  4a4f42c, then removed, so nothing is committed): output `portable computed [19.0, 22.0, 43.0, 50.0]`.

  ```rust
  use cpufish::{GemmArgs, Registry, Trans};

  fn main() -> Result<(), cpufish::Error> {
      let registry = Registry::with_portable();
      let provider = registry.available().into_iter().next().expect("portable is always available");
      let (a, b) = ([1.0_f32, 2.0, 3.0, 4.0], [5.0_f32, 6.0, 7.0, 8.0]); // 2x2 row-major
      let mut c = [0.0_f32; 4];
      provider.gemm_f32(&mut GemmArgs {
          trans_a: Trans::No, trans_b: Trans::No, m: 2, n: 2, k: 2,
          alpha: 1.0, a: &a, lda: 2, b: &b, ldb: 2, beta: 0.0, c: &mut c, ldc: 2,
      })?;
      println!("{} computed {c:?}", provider.id());
      Ok(())
  }
  ```

  Ease observation (facts, not a ruling): a 2x2 multiply takes a 13-field struct (`GemmArgs`) with explicit leading
  dimensions and transposes; there is no `provider.gemm(a, b)` convenience. That is the 80% case for 6.24 and is not
  yet one obvious short path. A vendor provider needs its library installed and its own crate (`cpufish-aocl` etc.), so the
  README's first example can only be `portable` today.
- **Does EXPECTATIONS 6.24 apply to cpufish?** UNVERIFIED. 6.24's text says "*Ant projects"; the word "Ant" appears nowhere in
  this repo and `PORTFOLIO-ROADMAP.md` lists CPUFish as Fuel's CPU backend lane, not under an Ant name. The rule's ground
  ("developer-facing") does fit a crate consumers will call. PM/pm-kernels to say. **The trial was NOT run:** an agent
  given only today's README has no example to reach, so the result would be a certain fail and say nothing new. Run it
  (Sonnet sub-agent, model pinned, README only, must reach `portable computed [19.0, 22.0, 43.0, 50.0]`) after the example
  lands.

## 2. Licences, NOTICE, and what is linked vs shipped

- **Our files:** `LICENSE-MIT` and `LICENSE-APACHE` exist at the repo root; every crate declares `MIT OR Apache-2.0`.
  **No `NOTICE` file** exists (`git ls-files` lists none). **GAP (packaging):** `cargo package --list` (run offline with
  `--allow-dirty`; `publish = false` did not stop it) shows only the root crate `cpufish` carries `LICENSE-MIT` and
  `LICENSE-APACHE`; `cpufish-sys`, `-aocl`, `-onemkl`, `-openblas` and `-bench` carry **no licence file**. Only
  `cpufish` and `cpufish-sys` are the intended first publish (the others are `publish = false` and see section 7).
- **Does any crate ship library bytes? No.** Package contents (`cargo package --list`): `cpufish` and every subcrate are
  `Cargo.toml`, `src/`, `tests/`, and for the root crate also the docs, `tools/install-openblas.ps1`, CI and
  `HANDOFF.md`/`ROADMAP.md` (no `exclude` is set; whether to ship those is a call). No `.dll/.so/.lib/.a/.dylib` is in
  any list. The dependency crates the providers use also ship none: `aocl-blas-sys 0.2.5` (12 files),
  `aocl-build 0.2.5` (10), `onemkl-sys 0.5.0` (9), read from the registry sources with the extension filter above (0 binary files;
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

## 3. Crate metadata (from each `Cargo.toml` at 4a4f42c)

All six crates: `version = 0.0.30`, `edition = 2024`, `rust-version = 1.95`, `license = "MIT OR Apache-2.0"`,
`repository = https://github.com/ThinkersJournal/cpufish`, `publish = false`, and a `description`.

| Crate | description | readme | keywords | categories | homepage/docs |
|---|---|---|---|---|---|
| `cpufish` | yes | **no key** (cargo auto-finds `README.md`? UNVERIFIED) | none | none | none |
| `cpufish-sys` | yes | none; no README file in the crate dir | none | none | none |
| `cpufish-aocl` | yes | none | none | none | none |
| `cpufish-onemkl` | yes | none | none | none | none |
| `cpufish-openblas` | yes | none | none | none | none |
| `cpufish-bench` | yes | file exists, no key | none | none | none |

**GAP:** no `keywords`, `categories`, `readme`, `homepage` or `documentation` anywhere. Only the root crate has a README
file; `cpufish-sys` (the other intended first publish) has none, so its crates.io page would be empty. All of `publish =
false` must be lifted per crate at publish time (the PM's call), and the path dependencies already carry a `version`
(e.g. `cpufish = { path = "../..", version = "0.0.30" }`), which is the publishable form.

## 4. MSRV stated and tested

- **Stated:** `rust-version = "1.95"` in all six crates.
- **Tested in CI: no.** `ci.yml` has no MSRV job; every job uses `toolchain: stable`. **GAP.**
- **Measured by hand today:** `cargo +1.94.0 check -p cpufish --offline` refuses with "rustc 1.94.0 is not supported ...
  requires rustc 1.95" (cargo enforces the field, so 1.94 cannot even start); `cargo +1.96 check -p cpufish -p cpufish-sys
  --locked` finishes. **1.95 itself is not installed on this box, so 1.95 exactly is UNVERIFIED**, and I did not install it
  (no system installs). Whether 1.95 is the true floor (versus an overstated one) is unmeasured: edition 2024 needs 1.85,
  and what needs 1.95 is not recorded here.

## 5. No path or git dependencies

- `git grep` for a dependency table line with `path =` or `git =` in `*.toml`: 8 hits, **all** workspace-internal
  `cpufish*` crates, each with a `version =` (list in the PR body). Zero hits name another repo or an external path.
  Control: the same pattern finds those 8, so it matches the syntax in use.
- `Cargo.lock`: 92 `source = "registry+https://github.com/rust-lang/crates.io-index"` entries, 0 `git+` sources, 98 `name =`
  entries (the 6 workspace crates are the 6 without a `source`). Every external dependency is a crates.io version, including
  `aocl-*` 0.2.5 and `onemkl` 0.5.0 (the `ciresnave`/ThinkersJournal crates are on crates.io, per CLAUDE.md section 9).
- Not checked: that each of those crates.io versions is the **latest** (rule: "all dependencies on their most recent versions"):
  last measured by the dependency-take PRs #29-#31 today; `libloading` 0.8.9 is also in the lock (a transitive second
  version; cargo-deny `multiple-versions = "warn"`).

## 6. Yank policy

- **No cpufish-specific yank policy exists in this repo** (`grep` of `CHANGELOG.md` and `ROADMAP.md` for "yank": 0 hits;
  control: the same search finds "placeholder" and "publish").
- Portfolio facts read: CireSnave 2026-10-11 (`CIRESNAVE-EXPECTATIONS.md` section 6.11, verbatim there): *"We can always
  yank anything that is wrong after the fact."* — the PM may publish any release without asking; yanks have been used
  elsewhere (auth-framework 0.4.2, item 32). Proposed default for cpufish (a proposal, not a ruling): a version is yanked only for
  a correctness bug that silently gives wrong numbers, a security problem, or a licence/provenance error; an ordinary bug is
  fixed forward in a patch release; each yank is recorded in `CHANGELOG.md` with the reason and the fixing version.

## 7. What the 0.0.0 placeholder reserved vs the workspace's crate names

Read from the crates.io API on 2026-11-27 (control: both found, four 404):

| Workspace crate | On crates.io | Note |
|---|---|---|
| `cpufish` | **0.0.0**, created 2026-10-30 17:40Z | reserved; no repository set on the placeholder |
| `cpufish-sys` | **0.0.0**, created 2026-10-30 17:40Z | reserved; no repository set |
| `cpufish-aocl` | 404 | **name not reserved** |
| `cpufish-onemkl` | 404 | **name not reserved** |
| `cpufish-openblas` | 404 | **name not reserved** |
| `cpufish-bench` | 404 | **name not reserved**; `publish = false`, a binary harness, probably never published |

**GAP (decision):** four workspace names are unreserved, and a vendor provider crate cannot be a path dependency of a published
`cpufish` consumer, so any release that has users of `-aocl`, `-onemkl` or `-openblas` needs those names reserved first (or
those crates kept unpublished and the README saying so). Also: the placeholder's repository field is empty (cannot be changed
without a release).

## Summary of gaps for the PM

1. No README hello-world; no compiled example in CI (section 1). 6.24 applicability UNVERIFIED; trial not run.
2. Subcrates ship no licence file; no `NOTICE`; AMD's EULA not read (section 2). cpufish ships no vendor bytes.
3. No keywords/categories/readme; no README for `cpufish-sys` (section 3).
4. No MSRV job in CI; 1.95 itself untested (section 4).
5. Four names unreserved (section 7). No cpufish yank policy written (section 6).
Item 5 of the TASK (no path/git deps, every dep on a published version) is PASS apart from the not-re-measured "latest".
