# Native libraries: where they live and how CPUFish finds them

Approved by CireSnave 2026-10-31 (relayed by the PM): one shared convention for the vendor math libraries CPUFish
loads (AOCL, oneMKL, OpenBLAS), and permission for CPUFish to load OpenBLAS. This note is the convention; the
OpenBLAS installer is `tools/install-openblas.ps1`; the loader order is in `crates/cpufish-openblas/src/lib.rs`
(`candidates`, unit-tested).

## The convention

1. **One admin-owned, read-only location per library and version.** A standard account can read and run it, not
   change it. No per-user copies, no copies inside a repo or a scratch directory.
2. **A pinned version, with the download's SHA-256 recorded** here and in the install script. Anything installed
   is checked against the pin before it is extracted or placed.
3. **Never on the system or user `PATH`.** A directory on `PATH` changes what every other program loads (and lets a
   planted DLL win a search). CPUFish finds the library by an explicit setting or by the fixed path below, never by
   searching.
4. **Fresh machine:** `tools/install-openblas.ps1` downloads the pinned zips and verifies the hashes (an elevated
   step, below). The loader itself never downloads or extracts: network and archive code does not belong in a
   math-library crate, and a loader that fetches on first use would run unreviewed bytes without an admin's say.

## Per library

| Library | Location | Pinned | How CPUFish finds it | Status |
|---|---|---|---|---|
| OpenBLAS | `%ProgramFiles%\OpenBLAS\0.3.34\lp64\libopenblas.dll` and `...\ilp64\libopenblas.dll` | 0.3.34 | run-time loader (`cpufish-sys`), order below | Linux/LP64 verified by CI; **Windows and ILP64 UNVERIFIED** until the install is done and tests run against it |
| AOCL | `%ProgramFiles%\AMD\AOCL-Windows` (the AMD installer's default) | 5.3.2 on the dev box (version on the runner **UNVERIFIED**) | link-time (`aocl-blas-sys`); the OS loader needs `...\amd-blis\lib\LP64` on the **process** `PATH` | installer hash **not recorded** (PM to supply); the directory's ACL **UNVERIFIED** |
| oneMKL | `%ProgramFiles(x86)%\Intel\oneAPI\2026.1` (the oneAPI installer's default) | 2026.1 (oneAPI) | link-time (`onemkl`); needs `...\bin` on the **process** `PATH` for `libiomp5md.dll` and the MKL DLLs | installer hash **not recorded** (PM to supply); the directory's ACL **UNVERIFIED** |

OpenBLAS pins (zips from `https://github.com/OpenMathLib/OpenBLAS/releases/download/v0.3.34/`; SHA-256 verified by
the PM against the OpenMathLib release API, and independently recomputed by this lane on 2026-10-31 from a fresh
download by `install-openblas.ps1 -VerifyOnly`):

| Zip | Bytes | SHA-256 |
|---|---|---|
| `OpenBLAS-0.3.34-x64.zip` (32-bit ints, `lp64`) | 40860061 | `e9cb6134541f36c27346d5fc5995652f060fba227cebbbabcbda5a5a44d7c76b` |
| `OpenBLAS-0.3.34-x64-64.zip` (64-bit ints, `ilp64`) | 40388456 | `55a5ff6f9e734a1d3caf84e79f2f8209ec34353e312c4d493853d4029ce4e0a9` |

The two builds each ship `bin\libopenblas.dll` under the **same file name**, so they can never share a directory.
(Correction to the PM's note: the `.def` files of both zips export the *same symbol names*, measured 2026-10-31, so
the symbols are not what differs. The width is told from `openblas_get_config`, which says `USE64BITINT` for the
ILP64 build, and `cpufish-sys` picks the matching call signature.)

AOCL and oneMKL are link-time providers (`docs/S2-AOCL-DESIGN.md`, option A): the program cannot start without
their DLLs, so the directory must be on the `PATH` of *that process*. Set it per process (the `aocl.yml` job does it
with `GITHUB_PATH`, which lasts one job; interactively, per shell), never machine-wide.

## Loader search order (OpenBLAS)

1. The path passed to `OpenBlas::with_path` (the caller's explicit choice).
2. The environment variable `CPUFISH_OPENBLAS`. **On Windows it must be an absolute path**; a bare name is refused,
   because a bare name is searched along `PATH` and the current directory.
3. Platform default. **Windows:** `%ProgramFiles%\OpenBLAS\0.3.34\lp64\libopenblas.dll`, then `...\ilp64\...`; no bare
   names and no `PATH`. **Linux/macOS:** the system loader's names (`libopenblas.so.0`, `libopenblas.so`,
   `libopenblas.dylib`), i.e. the distribution's own package, resolved by the system's loader configuration.

Nothing is found, loaded or changed until the provider is used; an absent library is
`ProbeError::Unavailable`, never a link error.

## Installing OpenBLAS (hand to CireSnave; do not run unattended)

From an **elevated** PowerShell, in a checkout of this repo at a commit CireSnave has read:

```powershell
# 1. Optional dry run, no admin needed: downloads both zips and checks the hashes, installs nothing.
powershell -ExecutionPolicy Bypass -File .\tools\install-openblas.ps1 -VerifyOnly

# 2. The install (elevated). Downloads, checks BOTH SHA-256 values, only then extracts bin/libopenblas.dll by exact
#    entry name, copies it to Program Files, sets the ACL, and reads everything back.
powershell -ExecutionPolicy Bypass -File .\tools\install-openblas.ps1
```

What step 2 does, in order, and stops at the first failure:

1. Downloads `OpenBLAS-0.3.34-x64.zip` and `...-x64-64.zip` into a new temp directory (or copies them from `-ZipDir`).
2. Computes SHA-256 of both and compares to the pins in the script. **Any mismatch throws before anything is
   extracted or written to Program Files.** (Negative control run 2026-10-31: junk files with the right names are
   rejected, and `Program Files` is untouched.)
3. Extracts only `bin/libopenblas.dll` from each zip, to staging files, using no path taken from the archive.
4. Copies them to `%ProgramFiles%\OpenBLAS\0.3.34\lp64\` and `...\ilp64\`, with a `MANIFEST.txt` beside each (source
   URL, zip hash, DLL hash, time). It refuses to overwrite an existing file whose content differs.
5. `icacls`: owner Administrators; Administrators and SYSTEM full control; Users read and execute only; inheritance
   removed.
6. **Read-back:** re-hashes each installed DLL against its manifest, checks every file and directory's owner and
   that no Users/Everyone/Authenticated Users entry grants write, modify, delete or change-permissions, prints the
   ACL, and checks no OpenBLAS directory is on the machine or user `PATH`. It ends with `OK: ...` or lists what failed
   and throws.

The script does not load the DLL. After it ends with `OK`, tell the CPUFish lane; then, and only then, the lane runs
`cargo test -p cpufish-openblas` against that install (a standard-user run, `CPUFISH_OPENBLAS_REQUIRED=1`), which is
what turns "Windows UNVERIFIED" into a measured result.

## What is and is not verified (2026-10-31)

- Verified: the pins match fresh downloads (PowerShell 5.1); a mismatch is rejected; the search order's unit tests
  pass; the OpenBLAS provider passes the conformance suite against distribution OpenBLAS 0.3.26 (LP64) on hosted
  Linux.
- **Not verified:** the install step itself (it needs an elevation this lane does not have and was told not to use);
  the read-back on a real install; loading the Windows DLLs, either width; ILP64 anywhere; any claim about AOCL's and
  oneMKL's installer hashes or directory permissions; macOS.
