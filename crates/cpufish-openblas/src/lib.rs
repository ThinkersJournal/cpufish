// SPDX-License-Identifier: MIT OR Apache-2.0
//! OpenBLAS provider for CPUFish, S5 in `docs/S5-OPENBLAS-DESIGN.md`.
//!
//! The library is loaded at run time by `cpufish-sys`, so this crate builds and links without OpenBLAS and an
//! absent library is [`ProbeError::Unavailable`]. Where to look, in order: the path given to
//! [`OpenBlas::with_path`], the `CPUFISH_OPENBLAS` environment variable, then the platform default (on Windows the
//! install directory of `docs/NATIVE-LIBS.md`, never `PATH`).

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::OnceLock;

use cpufish::{Capability, Config, Dtype, Error, GemmArgs, Op, ProbeError, Provider, Trans};
use cpufish_sys::{GemmCall, Library, Scalar};

const ID: &str = "openblas";
/// Environment variable naming the OpenBLAS shared library (an absolute path on Windows).
pub const ENV_VAR: &str = "CPUFISH_OPENBLAS";
const NOTES: &str =
    "OpenBLAS cblas gemm, loaded at run time; threads per OPENBLAS_NUM_THREADS/OMP_NUM_THREADS";
const CAPS: [Capability; 2] = [
    Capability {
        op: Op::Gemm,
        dtype: Dtype::F32,
        notes: NOTES,
    },
    Capability {
        op: Op::Gemm,
        dtype: Dtype::F64,
        notes: NOTES,
    },
];

/// The OpenBLAS release the install convention in `docs/NATIVE-LIBS.md` pins.
pub const PINNED_VERSION: &str = "0.3.34";

#[cfg(target_os = "macos")]
const DEFAULT_NAMES: &[&str] = &["libopenblas.dylib", "libopenblas.0.dylib"];
#[cfg(not(any(windows, target_os = "macos")))]
const DEFAULT_NAMES: &[&str] = &["libopenblas.so.0", "libopenblas.so"];

/// Where to look, in order (`docs/NATIVE-LIBS.md`): the explicit path, then `CPUFISH_OPENBLAS`, then the platform
/// default. On Windows the default is the admin-owned install directory under `Program Files`, never a bare name:
/// a bare name would be searched along `PATH` and the current directory, which the convention avoids, and
/// `CPUFISH_OPENBLAS` must be an absolute path for the same reason.
fn candidates(
    windows: bool,
    explicit: Option<&OsStr>,
    env: Option<OsString>,
    program_files: Option<OsString>,
) -> Result<Vec<OsString>, String> {
    if let Some(p) = explicit {
        return Ok(vec![p.to_owned()]);
    }
    if let Some(p) = env.filter(|p| !p.is_empty()) {
        if windows && !Path::new(&p).is_absolute() {
            return Err(format!(
                "{ENV_VAR} must be an absolute path to the library on Windows, got {p:?}"
            ));
        }
        return Ok(vec![p]);
    }
    if windows {
        let Some(pf) = program_files else {
            return Err("ProgramFiles is not set, so the install directory is unknown".to_owned());
        };
        let root = Path::new(&pf).join("OpenBLAS").join(PINNED_VERSION);
        // lp64 first (32-bit ints); the loader accepts either width.
        return Ok(["lp64", "ilp64"]
            .iter()
            .map(|w| root.join(w).join("libopenblas.dll").into_os_string())
            .collect());
    }
    #[cfg(not(windows))]
    {
        Ok(DEFAULT_NAMES.iter().map(OsString::from).collect())
    }
    #[cfg(windows)]
    {
        Ok(Vec::new())
    }
}

/// The OpenBLAS provider. The library is opened on first use and kept for the life of the value.
#[derive(Debug, Default)]
pub struct OpenBlas {
    path: Option<OsString>,
    lib: OnceLock<Result<Library, String>>,
}

impl OpenBlas {
    /// Create the provider. Nothing is loaded until a method needs the library.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create the provider for one specific library file or name, ignoring `CPUFISH_OPENBLAS`.
    pub fn with_path(path: impl Into<OsString>) -> Self {
        Self {
            path: Some(path.into()),
            lib: OnceLock::new(),
        }
    }

    fn library(&self) -> Result<&Library, String> {
        self.lib
            .get_or_init(|| {
                let candidates = candidates(
                    cfg!(windows),
                    self.path.as_deref(),
                    std::env::var_os(ENV_VAR),
                    std::env::var_os("ProgramFiles"),
                )?;
                Library::open(&candidates).map_err(|e| e.to_string())
            })
            .as_ref()
            .map_err(Clone::clone)
    }

    fn lib_or_backend(&self) -> Result<&Library, Error> {
        self.library().map_err(|message| Error::Backend {
            provider: ID,
            message,
        })
    }
}

fn gemm<T: Scalar>(lib: &Library, args: &mut GemmArgs<'_, T>) -> Result<(), Error> {
    // Our own check first; the loader re-checks buffers as a second line.
    args.validate()?;
    let mut call = GemmCall {
        trans_a: args.trans_a == Trans::Yes,
        trans_b: args.trans_b == Trans::Yes,
        m: args.m,
        n: args.n,
        k: args.k,
        alpha: args.alpha,
        a: args.a,
        lda: args.lda,
        b: args.b,
        ldb: args.ldb,
        beta: args.beta,
        c: &mut *args.c,
        ldc: args.ldc,
    };
    lib.gemm(&mut call).map_err(|e| Error::Backend {
        provider: ID,
        message: e.to_string(),
    })
}

impl Provider for OpenBlas {
    fn id(&self) -> &'static str {
        ID
    }

    fn version(&self) -> Option<String> {
        self.library().ok().and_then(Library::version)
    }

    fn probe(&self) -> Result<(), ProbeError> {
        self.library().map_err(ProbeError::Unavailable)?;
        let a = [1.0_f32, 2.0, 3.0, 4.0];
        let b = [1.0_f32, 0.0, 0.0, 1.0];
        let mut c = [0.0_f32; 4];
        let mut args = GemmArgs {
            trans_a: Trans::No,
            trans_b: Trans::No,
            m: 2,
            n: 2,
            k: 2,
            alpha: 1.0,
            a: &a,
            lda: 2,
            b: &b,
            ldb: 2,
            beta: 0.0,
            c: &mut c,
            ldc: 2,
        };
        self.gemm_f32(&mut args)
            .map_err(|e| ProbeError::Unavailable(e.to_string()))?;
        if c != a {
            return Err(ProbeError::WrongAnswer(format!("{c:?} != {a:?}")));
        }
        Ok(())
    }

    fn capabilities(&self) -> &[Capability] {
        &CAPS
    }

    fn effective_config(&self) -> Config {
        match self.library() {
            Ok(lib) => Config {
                threads: lib.threads(),
                isa: lib.corename(),
            },
            Err(_) => Config {
                threads: None,
                isa: None,
            },
        }
    }

    fn set_threads(&self, n: usize) -> Result<(), Error> {
        let value = std::ffi::c_int::try_from(n)
            .ok()
            .filter(|&v| v >= 1)
            .ok_or_else(|| Error::InvalidArgument(format!("thread count {n} out of range")))?;
        if self.lib_or_backend()?.set_threads(value) {
            Ok(())
        } else {
            Err(Error::Backend {
                provider: ID,
                message: "this OpenBLAS has no openblas_set_num_threads".to_owned(),
            })
        }
    }

    fn gemm_f32(&self, args: &mut GemmArgs<'_, f32>) -> Result<(), Error> {
        gemm(self.lib_or_backend()?, args)
    }

    fn gemm_f64(&self, args: &mut GemmArgs<'_, f64>) -> Result<(), Error> {
        gemm(self.lib_or_backend()?, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_default_is_the_install_dir_and_never_a_bare_name() {
        let pf = OsString::from(r"C:\Program Files");
        let c = candidates(true, None, None, Some(pf.clone())).unwrap();
        let want = |w: &str| {
            Path::new(&pf)
                .join("OpenBLAS")
                .join(PINNED_VERSION)
                .join(w)
                .join("libopenblas.dll")
                .into_os_string()
        };
        assert_eq!(c, vec![want("lp64"), want("ilp64")]);
        assert!(c.iter().all(|p| Path::new(p).components().count() > 1));
        assert!(candidates(true, None, None, None).is_err());
    }

    #[test]
    fn explicit_then_env_win_over_the_default() {
        let explicit = OsString::from("x");
        let c = candidates(true, Some(&explicit), Some("y".into()), None).unwrap();
        assert_eq!(c, vec![OsString::from("x")]);
        let c = candidates(false, None, Some("/opt/y.so".into()), None).unwrap();
        assert_eq!(c, vec![OsString::from("/opt/y.so")]);
        // An empty variable counts as unset.
        assert!(candidates(true, None, Some("".into()), Some("PF".into())).is_ok());
    }

    #[test]
    fn windows_env_var_must_be_absolute() {
        let e = candidates(true, None, Some("libopenblas.dll".into()), None).unwrap_err();
        assert!(e.contains(ENV_VAR), "{e}");
        // `Path::is_absolute` is platform-specific, so only assert the accepted form where it applies.
        #[cfg(windows)]
        assert!(candidates(true, None, Some(r"C:\x\libopenblas.dll".into()), None).is_ok());
    }

    /// With no library at all the provider is `Unavailable` and every call is a `Backend` error, never a crash.
    #[test]
    fn absent_library_is_unavailable() {
        let p = OpenBlas::with_path("cpufish-no-such-library-xyz");
        assert!(matches!(p.probe(), Err(ProbeError::Unavailable(_))));
        assert_eq!(p.version(), None);
        assert_eq!(p.effective_config().threads, None);
        let mut c = [0.0_f32; 1];
        let mut args = GemmArgs {
            trans_a: Trans::No,
            trans_b: Trans::No,
            m: 1,
            n: 1,
            k: 1,
            alpha: 1.0,
            a: &[1.0],
            lda: 1,
            b: &[1.0],
            ldb: 1,
            beta: 0.0,
            c: &mut c,
            ldc: 1,
        };
        assert!(matches!(p.gemm_f32(&mut args), Err(Error::Backend { .. })));
        assert!(p.set_threads(2).is_err());
    }

    /// Asserts the integer width the loader derives from the config string. Runs only when
    /// `CPUFISH_OPENBLAS_EXPECT_WIDTH` is `lp64` or `ilp64` and `CPUFISH_OPENBLAS` names the library, so a
    /// conformance pass can be tied to the width it ran at.
    #[test]
    fn loader_reports_the_expected_int_width() {
        use cpufish_sys::IntWidth;
        let (Some(want), Some(path)) = (
            std::env::var_os("CPUFISH_OPENBLAS_EXPECT_WIDTH"),
            std::env::var_os("CPUFISH_OPENBLAS"),
        ) else {
            return;
        };
        let want = match want.to_str() {
            Some("lp64") => IntWidth::Lp64,
            Some("ilp64") => IntWidth::Ilp64,
            other => panic!("CPUFISH_OPENBLAS_EXPECT_WIDTH must be lp64 or ilp64, got {other:?}"),
        };
        let lib = Library::open(&[path]).expect("open");
        eprintln!("config {:?} width {:?}", lib.config(), lib.int_width());
        assert_eq!(lib.int_width(), want);
    }

    /// The real library, found the normal way. `CPUFISH_OPENBLAS_REQUIRED=1` (set by the CI jobs that install
    /// OpenBLAS) makes a missing library a failure; elsewhere it is reported and the absent path is what the test
    /// above proves.
    #[test]
    fn conformance_against_installed_openblas() {
        let p = OpenBlas::new();
        match p.probe() {
            Ok(()) => {
                eprintln!(
                    "openblas {:?} config {:?}",
                    p.version(),
                    p.effective_config()
                );
                cpufish::conformance::check_provider(&p).expect("conformance");
            }
            Err(ProbeError::Unavailable(why))
                if std::env::var_os("CPUFISH_OPENBLAS_REQUIRED").is_none() =>
            {
                eprintln!("OpenBLAS not found, conformance not run: {why}");
            }
            Err(e) => panic!("{e}"),
        }
    }
}
