//! Intel oneAPI MKL provider for CPUFish (S3), over the `onemkl` crate's cblas gemm.
//!
//! Link-time provider like `cpufish-aocl`: building and starting a program that includes it needs oneMKL.

use cpufish::{Capability, Config, Dtype, Error, GemmArgs, Op, ProbeError, Provider, Trans};
use onemkl::blas::BlasScalar;
use onemkl::blas::level3::gemm as mkl_gemm;
use onemkl::{Layout, MatrixMut, MatrixRef, Transpose};

const ID: &str = "onemkl";
const NOTES: &str = "oneMKL cblas gemm; threading layer chosen by cargo feature";
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

/// The oneMKL provider.
#[derive(Clone, Copy, Debug, Default)]
pub struct OneMkl;

impl OneMkl {
    /// Create the provider. Nothing is loaded or changed until a method is called.
    pub fn new() -> Self {
        Self
    }
}

fn trans(t: Trans) -> Transpose {
    match t {
        Trans::No => Transpose::NoTrans,
        Trans::Yes => Transpose::Trans,
    }
}

fn backend(e: impl std::fmt::Display) -> Error {
    Error::Backend {
        provider: ID,
        message: e.to_string(),
    }
}

fn gemm<T: BlasScalar + PartialEq + Default + std::ops::Mul<Output = T>>(
    args: &mut GemmArgs<'_, T>,
) -> Result<(), Error> {
    args.validate()?;
    if args.m == 0 || args.n == 0 {
        return Ok(());
    }
    if args.k == 0 {
        // BLAS defines this as `C := beta * C`; `beta == 0` writes zeros and never reads C. Done here so
        // the result does not depend on how the wrapper sizes the (unused) A and B at k == 0.
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
    let (ar, ac) = match args.trans_a {
        Trans::No => (args.m, args.k),
        Trans::Yes => (args.k, args.m),
    };
    let (br, bc) = match args.trans_b {
        Trans::No => (args.k, args.n),
        Trans::Yes => (args.n, args.k),
    };
    let a = MatrixRef::with_lda(args.a, ar, ac, Layout::RowMajor, args.lda).map_err(backend)?;
    let b = MatrixRef::with_lda(args.b, br, bc, Layout::RowMajor, args.ldb).map_err(backend)?;
    let mut c =
        MatrixMut::with_lda(args.c, args.m, args.n, Layout::RowMajor, args.ldc).map_err(backend)?;
    mkl_gemm(
        trans(args.trans_a),
        trans(args.trans_b),
        args.alpha,
        &a,
        &b,
        args.beta,
        &mut c,
    )
    .map_err(backend)
}

impl Provider for OneMkl {
    fn id(&self) -> &'static str {
        ID
    }

    fn version(&self) -> Option<String> {
        Some(onemkl::service::version_string())
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
        // oneMKL exposes the processor family it dispatches for, not the vector-ISA tier in force.
        Config {
            threads: usize::try_from(onemkl::service::max_threads()).ok(),
            isa: Some(onemkl::service::version().processor),
        }
    }

    fn set_threads(&self, n: usize) -> Result<(), Error> {
        let value = i32::try_from(n)
            .ok()
            .filter(|&v| v >= 1)
            .ok_or_else(|| Error::InvalidArgument(format!("thread count {n} out of range")))?;
        onemkl::service::set_num_threads(value);
        Ok(())
    }

    fn gemm_f32(&self, args: &mut GemmArgs<'_, f32>) -> Result<(), Error> {
        gemm(args)
    }

    fn gemm_f64(&self, args: &mut GemmArgs<'_, f64>) -> Result<(), Error> {
        gemm(args)
    }
}
