//! AOCL (AMD BLIS) provider for CPUFish, S2 in `docs/S2-AOCL-DESIGN.md`.
//!
//! This is a link-time provider: building a program that includes it needs AOCL, and starting it needs the
//! AOCL DLL/shared library to be findable (on Windows also `libiomp5md.dll` for the multi-threaded BLIS).

use std::ffi::CStr;

use aocl_blas::Scalar;
use aocl_types::{Layout, Trans as AoclTrans};
use cpufish::{Capability, Config, Dtype, Error, GemmArgs, Op, ProbeError, Provider, Trans};

const ID: &str = "aocl";
const NOTES: &str = "AOCL BLIS cblas gemm; threads per BLIS_NUM_THREADS/OMP_NUM_THREADS";
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

/// The AOCL provider.
#[derive(Clone, Copy, Debug, Default)]
pub struct Aocl;

impl Aocl {
    /// Create the provider. Nothing is loaded or changed until a method is called.
    pub fn new() -> Self {
        Self
    }
}

fn trans(t: Trans) -> AoclTrans {
    match t {
        Trans::No => AoclTrans::No,
        Trans::Yes => AoclTrans::T,
    }
}

fn gemm<T: Scalar + PartialEq + Default + std::ops::Mul<Output = T>>(
    args: &mut GemmArgs<'_, T>,
) -> Result<(), Error> {
    // Our own check first; the wrapper re-checks shapes as a second line.
    args.validate()?;
    if args.m == 0 || args.n == 0 {
        return Ok(());
    }
    if args.k == 0 {
        // BLAS defines this as `C := beta * C`; `aocl-blas`'s shape check wants a longer (unused) A and B here
        // than the Provider contract promises, so do it ourselves. `beta == 0` writes zeros and never reads C.
        for row in args.c.chunks_mut(args.ldc).take(args.m) {
            for x in &mut row[..args.n] {
                *x = if args.beta == T::default() {
                    T::default()
                } else {
                    *x * args.beta
                };
            }
        }
        return Ok(());
    }
    T::gemm(
        Layout::RowMajor,
        trans(args.trans_a),
        trans(args.trans_b),
        args.m,
        args.n,
        args.k,
        args.alpha,
        args.a,
        args.lda,
        args.b,
        args.ldb,
        args.beta,
        args.c,
        args.ldc,
    )
    .map_err(|e| Error::Backend {
        provider: ID,
        message: e.to_string(),
    })
}

/// Copy a NUL-terminated string BLIS returns, or `None` for a null pointer.
///
/// # Safety
/// `p` is null or points at a NUL-terminated string that outlives the call.
unsafe fn c_string(p: *const std::os::raw::c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        // SAFETY: caller contract.
        Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
    }
}

impl Provider for Aocl {
    fn id(&self) -> &'static str {
        ID
    }

    fn version(&self) -> Option<String> {
        // SAFETY: returns a pointer to a static NUL-terminated string.
        unsafe { c_string(aocl_blas_sys::bli_info_get_version_str()) }
    }

    fn probe(&self) -> Result<(), ProbeError> {
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
        // SAFETY: both are plain queries; `bli_arch_string` returns a static NUL-terminated string.
        let (threads, isa) = unsafe {
            (
                usize::try_from(aocl_blas_sys::bli_thread_get_num_threads()).ok(),
                c_string(aocl_blas_sys::bli_arch_string(
                    aocl_blas_sys::bli_arch_query_id(),
                )),
            )
        };
        Config { threads, isa }
    }

    fn gemm_f32(&self, args: &mut GemmArgs<'_, f32>) -> Result<(), Error> {
        gemm(args)
    }

    fn gemm_f64(&self, args: &mut GemmArgs<'_, f64>) -> Result<(), Error> {
        gemm(args)
    }
}
