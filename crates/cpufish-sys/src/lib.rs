// SPDX-License-Identifier: MIT OR Apache-2.0
//! Run-time loader for CBLAS-ABI math libraries, S5 in `docs/S5-OPENBLAS-DESIGN.md`.
//!
//! [`Library::open`] loads a shared library with `libloading` and resolves the few symbols CPUFish uses, so a
//! missing library or symbol is an ordinary [`LoadError`] and never a link error for the program. This crate owns
//! the `unsafe`; its API is safe and bounds-checks every buffer before the foreign call.
//!
//! Only OpenBLAS's introspection symbols (`openblas_get_config`, ...) are looked up beyond plain CBLAS, so today
//! this loads OpenBLAS. Both integer widths are handled: the official OpenBLAS builds name their symbols the same
//! for LP64 and ILP64, so the width is read from the build's config string (`USE64BITINT`) and the matching
//! call signature is used.

use std::ffi::{CStr, OsStr, c_char, c_int};

use libloading::Library as Dl;

const ROW_MAJOR: c_int = 101;
const NO_TRANS: c_int = 111;
const TRANS: c_int = 112;

/// Why a library could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// None of the candidate library names could be opened.
    #[error("could not load any of {tried:?}: {last}")]
    NotFound {
        /// The candidates tried, in order.
        tried: Vec<String>,
        /// The loader's message for the last one.
        last: String,
    },
    /// The library loaded but lacks a symbol we need.
    #[error("library `{library}` has no symbol `{symbol}`")]
    MissingSymbol {
        /// The library that was opened.
        library: String,
        /// The symbol that was not found.
        symbol: &'static str,
    },
}

/// Why a gemm call was refused before reaching the library.
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    /// A dimension, leading dimension or buffer length is inconsistent.
    #[error("{0}")]
    Shape(String),
    /// A value does not fit the library's integer type (32-bit for an LP64 build).
    #[error("{0} does not fit the library's integer width")]
    TooLarge(&'static str),
}

/// The integer width a library was built with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntWidth {
    /// 32-bit `blasint` (LP64).
    Lp64,
    /// 64-bit `blasint` (ILP64).
    Ilp64,
}

type Gemm<T, I> =
    unsafe extern "C" fn(c_int, c_int, c_int, I, I, I, T, *const T, I, *const T, I, T, *mut T, I);

#[derive(Clone, Copy)]
#[doc(hidden)]
pub enum Kernel<T> {
    Lp64(Gemm<T, i32>),
    Ilp64(Gemm<T, i64>),
}

/// A loaded CBLAS library.
pub struct Library {
    name: String,
    width: IntWidth,
    sgemm: Kernel<f32>,
    dgemm: Kernel<f64>,
    get_threads: Option<unsafe extern "C" fn() -> c_int>,
    set_threads: Option<unsafe extern "C" fn(c_int)>,
    corename: Option<unsafe extern "C" fn() -> *const c_char>,
    config: String,
    // Declared last: the function pointers above are only valid while this is loaded.
    _lib: Dl,
}

impl std::fmt::Debug for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Library")
            .field("name", &self.name)
            .field("width", &self.width)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// Copy a NUL-terminated string a library returned, or `None` for a null pointer.
///
/// # Safety
/// `p` is null or points at a NUL-terminated string that stays valid for the call.
unsafe fn c_string(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        // SAFETY: caller contract.
        Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
    }
}

/// A float type the loader can call gemm for: `f32` and `f64`.
pub trait Scalar: Copy + sealed::Sealed {}

mod sealed {
    use super::{Kernel, Library};

    pub trait Sealed: Sized {
        fn kernel(lib: &Library) -> Kernel<Self>;
    }

    impl Sealed for f32 {
        fn kernel(lib: &Library) -> Kernel<f32> {
            lib.sgemm
        }
    }

    impl Sealed for f64 {
        fn kernel(lib: &Library) -> Kernel<f64> {
            lib.dgemm
        }
    }
}

impl Scalar for f32 {}
impl Scalar for f64 {}

/// Number of elements a row-major `rows x cols` matrix with leading dimension `ld` spans.
fn span(rows: usize, cols: usize, ld: usize) -> Option<usize> {
    if rows == 0 || cols == 0 {
        Some(0)
    } else {
        (rows - 1).checked_mul(ld)?.checked_add(cols)
    }
}

fn check_matrix(
    name: &str,
    rows: usize,
    cols: usize,
    ld: usize,
    len: usize,
) -> Result<(), CallError> {
    if ld < cols.max(1) {
        return Err(CallError::Shape(format!("ld{name} {ld} < {}", cols.max(1))));
    }
    let need = span(rows, cols, ld).ok_or(CallError::TooLarge("matrix size"))?;
    if len < need {
        return Err(CallError::Shape(format!(
            "{name} has {len} elements, needs {need}"
        )));
    }
    Ok(())
}

fn conv<I: TryFrom<usize>>(v: usize, what: &'static str) -> Result<I, CallError> {
    I::try_from(v).map_err(|_| CallError::TooLarge(what))
}

/// Row-major gemm arguments in plain values, so this crate needs no knowledge of CPUFish's types.
pub struct GemmCall<'a, T> {
    /// Transpose A.
    pub trans_a: bool,
    /// Transpose B.
    pub trans_b: bool,
    /// Rows of op(A) and C.
    pub m: usize,
    /// Columns of op(B) and C.
    pub n: usize,
    /// Columns of op(A), rows of op(B).
    pub k: usize,
    /// Scale on the product.
    pub alpha: T,
    /// Matrix A.
    pub a: &'a [T],
    /// Leading dimension of A.
    pub lda: usize,
    /// Matrix B.
    pub b: &'a [T],
    /// Leading dimension of B.
    pub ldb: usize,
    /// Scale on C.
    pub beta: T,
    /// Matrix C, overwritten.
    pub c: &'a mut [T],
    /// Leading dimension of C.
    pub ldc: usize,
}

impl Library {
    /// Open the first of `candidates` that loads (a bare name searched by the OS, or a path) and resolve symbols.
    pub fn open<S: AsRef<OsStr>>(candidates: &[S]) -> Result<Self, LoadError> {
        let mut last = String::from("no candidates given");
        for cand in candidates {
            let name = cand.as_ref().to_string_lossy().into_owned();
            // SAFETY: loading a shared library runs its initialisers; that is the point of this loader and the
            // candidate is chosen by the caller, not derived from untrusted data here.
            match unsafe { Dl::new(cand.as_ref()) } {
                Ok(lib) => return Self::from_loaded(name, lib),
                Err(e) => last = format!("{name}: {e}"),
            }
        }
        Err(LoadError::NotFound {
            tried: candidates
                .iter()
                .map(|c| c.as_ref().to_string_lossy().into_owned())
                .collect(),
            last,
        })
    }

    fn from_loaded(name: String, lib: Dl) -> Result<Self, LoadError> {
        macro_rules! req {
            ($sym:literal, $ty:ty) => {{
                // SAFETY: the declared type is the C prototype from OpenBLAS's `cblas.h`; the pointer is copied
                // out and only used while `_lib` keeps the library loaded (it is a field of the same struct).
                let s: Result<libloading::Symbol<'_, $ty>, _> = unsafe { lib.get(concat!($sym, "\0").as_bytes()) };
                match s {
                    Ok(s) => *s,
                    Err(_) => {
                        return Err(LoadError::MissingSymbol {
                            library: name,
                            symbol: $sym,
                        });
                    }
                }
            }};
        }
        macro_rules! opt {
            ($sym:literal, $ty:ty) => {{
                // SAFETY: as for `req!`.
                let s: Result<libloading::Symbol<'_, $ty>, _> =
                    unsafe { lib.get(concat!($sym, "\0").as_bytes()) };
                s.ok().map(|s| *s)
            }};
        }

        let get_config = req!(
            "openblas_get_config",
            unsafe extern "C" fn() -> *const c_char
        );
        // SAFETY: returns a pointer to a static NUL-terminated string.
        let config = unsafe { c_string(get_config()) }.unwrap_or_default();
        let width = if config.contains("USE64BITINT") {
            IntWidth::Ilp64
        } else {
            IntWidth::Lp64
        };
        let (sgemm, dgemm) = match width {
            IntWidth::Lp64 => (
                Kernel::Lp64(req!("cblas_sgemm", Gemm<f32, i32>)),
                Kernel::Lp64(req!("cblas_dgemm", Gemm<f64, i32>)),
            ),
            IntWidth::Ilp64 => (
                Kernel::Ilp64(req!("cblas_sgemm", Gemm<f32, i64>)),
                Kernel::Ilp64(req!("cblas_dgemm", Gemm<f64, i64>)),
            ),
        };
        let get_threads = opt!("openblas_get_num_threads", unsafe extern "C" fn() -> c_int);
        let set_threads = opt!("openblas_set_num_threads", unsafe extern "C" fn(c_int));
        let corename = opt!(
            "openblas_get_corename",
            unsafe extern "C" fn() -> *const c_char
        );
        Ok(Self {
            name,
            width,
            sgemm,
            dgemm,
            get_threads,
            set_threads,
            corename,
            config,
            _lib: lib,
        })
    }

    /// The candidate that was opened.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The library's integer width, read from its config string.
    pub fn int_width(&self) -> IntWidth {
        self.width
    }

    /// OpenBLAS's build string, for example ` OpenBLAS 0.3.34 USE64BITINT DYNAMIC_ARCH ... MAX_THREADS=64`.
    pub fn config(&self) -> &str {
        &self.config
    }

    /// The version in the config string, if it has the `OpenBLAS x.y.z` form.
    pub fn version(&self) -> Option<String> {
        let mut words = self.config.split_whitespace();
        words.find(|w| *w == "OpenBLAS")?;
        words
            .next()
            .filter(|v| v.starts_with(|c: char| c.is_ascii_digit()))
            .map(str::to_owned)
    }

    /// The kernel set the library selected (for example `ZEN`, `HASWELL`, `SKYLAKEX`), if it reports one.
    pub fn corename(&self) -> Option<String> {
        // SAFETY: `openblas_get_corename` returns a pointer to a static NUL-terminated string.
        self.corename.and_then(|f| unsafe { c_string(f()) })
    }

    /// The library's current thread count, if it reports one.
    pub fn threads(&self) -> Option<usize> {
        // SAFETY: plain query with no arguments.
        self.get_threads
            .and_then(|f| usize::try_from(unsafe { f() }).ok())
    }

    /// Set the library's thread count (process-global). `false` if the library has no such setter.
    pub fn set_threads(&self, n: c_int) -> bool {
        match self.set_threads {
            Some(f) => {
                // SAFETY: plain setter taking an `int`.
                unsafe { f(n) };
                true
            }
            None => false,
        }
    }

    /// Row-major `C := alpha * op(A) * op(B) + beta * C`, after checking every dimension and buffer length.
    pub fn gemm<T: Scalar>(&self, g: &mut GemmCall<'_, T>) -> Result<(), CallError> {
        let (ra, ca) = if g.trans_a { (g.k, g.m) } else { (g.m, g.k) };
        let (rb, cb) = if g.trans_b { (g.n, g.k) } else { (g.k, g.n) };
        check_matrix("a", ra, ca, g.lda, g.a.len())?;
        check_matrix("b", rb, cb, g.ldb, g.b.len())?;
        check_matrix("c", g.m, g.n, g.ldc, g.c.len())?;
        let ta = if g.trans_a { TRANS } else { NO_TRANS };
        let tb = if g.trans_b { TRANS } else { NO_TRANS };
        match <T as sealed::Sealed>::kernel(self) {
            Kernel::Lp64(f) => {
                let (m, n, k) = (conv(g.m, "m")?, conv(g.n, "n")?, conv(g.k, "k")?);
                let (lda, ldb, ldc) = (
                    conv(g.lda, "lda")?,
                    conv(g.ldb, "ldb")?,
                    conv(g.ldc, "ldc")?,
                );
                // SAFETY: the pointers cover the spans checked above for the given leading dimensions, the
                // function pointer matches the build's integer width, and `c` is exclusively borrowed.
                unsafe {
                    f(
                        ROW_MAJOR,
                        ta,
                        tb,
                        m,
                        n,
                        k,
                        g.alpha,
                        g.a.as_ptr(),
                        lda,
                        g.b.as_ptr(),
                        ldb,
                        g.beta,
                        g.c.as_mut_ptr(),
                        ldc,
                    )
                };
            }
            Kernel::Ilp64(f) => {
                let (m, n, k) = (conv(g.m, "m")?, conv(g.n, "n")?, conv(g.k, "k")?);
                let (lda, ldb, ldc) = (
                    conv(g.lda, "lda")?,
                    conv(g.ldb, "ldb")?,
                    conv(g.ldc, "ldc")?,
                );
                // SAFETY: as above, with 64-bit integers.
                unsafe {
                    f(
                        ROW_MAJOR,
                        ta,
                        tb,
                        m,
                        n,
                        k,
                        g.alpha,
                        g.a.as_ptr(),
                        lda,
                        g.b.as_ptr(),
                        ldb,
                        g.beta,
                        g.c.as_mut_ptr(),
                        ldc,
                    )
                };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_library_is_an_error_not_a_crash() {
        let e = Library::open(&["cpufish-no-such-library-xyz"]).unwrap_err();
        assert!(matches!(e, LoadError::NotFound { .. }), "{e}");
        assert!(e.to_string().contains("cpufish-no-such-library-xyz"));
    }

    #[test]
    fn no_candidates_is_an_error() {
        let none: [&str; 0] = [];
        assert!(matches!(
            Library::open(&none),
            Err(LoadError::NotFound { .. })
        ));
    }

    #[test]
    fn span_and_checks() {
        assert_eq!(span(0, 5, 5), Some(0));
        assert_eq!(span(3, 4, 6), Some(2 * 6 + 4));
        assert_eq!(span(usize::MAX, 1, 2), None);
        assert!(check_matrix("a", 2, 3, 2, 6).is_err(), "ld < cols");
        assert!(check_matrix("a", 2, 3, 3, 5).is_err(), "too short");
        assert!(check_matrix("a", 2, 3, 3, 6).is_ok());
        assert!(check_matrix("a", 0, 3, 3, 0).is_ok());
    }

    /// A real library that is not CBLAS: loads fine, so the failure must be the missing symbol.
    #[test]
    fn library_without_symbols_reports_missing_symbol() {
        let candidates: &[&str] = if cfg!(windows) {
            &["kernel32.dll"]
        } else if cfg!(target_os = "macos") {
            &["/usr/lib/libSystem.B.dylib"]
        } else {
            &["libc.so.6"]
        };
        let e = Library::open(candidates).unwrap_err();
        assert!(
            matches!(
                e,
                LoadError::MissingSymbol {
                    symbol: "openblas_get_config",
                    ..
                }
            ),
            "{e}"
        );
    }
}
