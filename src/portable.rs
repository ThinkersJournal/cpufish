//! The always-available provider: pure Rust, through the `gemm` crate, single-threaded.
//!
//! Single-threaded on purpose: results are then reproducible run to run, which makes it a stable
//! reference for the vendor providers.

use crate::{Capability, Config, Dtype, Error, GemmArgs, Op, ProbeError, Provider, Trans};

/// The pure-Rust provider.
#[derive(Clone, Copy, Debug, Default)]
pub struct Portable;

const NOTES: &str = "gemm crate; single-threaded, runtime SIMD dispatch";
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

impl Portable {
    /// Create the provider.
    pub fn new() -> Self {
        Self
    }
}

impl Provider for Portable {
    fn id(&self) -> &'static str {
        "portable"
    }

    fn version(&self) -> Option<String> {
        None
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
        Config {
            threads: Some(1),
            isa: Some("runtime-detected by the gemm crate".to_owned()),
        }
    }

    fn set_threads(&self, n: usize) -> Result<(), Error> {
        if n == 1 {
            Ok(())
        } else {
            Err(Error::InvalidArgument(format!(
                "portable is single-threaded; cannot set {n} threads"
            )))
        }
    }

    fn gemm_f32(&self, args: &mut GemmArgs<'_, f32>) -> Result<(), Error> {
        run(args)
    }

    fn gemm_f64(&self, args: &mut GemmArgs<'_, f64>) -> Result<(), Error> {
        run(args)
    }
}

/// Bounds are checked by `validate`, which is what makes the unsafe call below sound.
fn run<T: Copy + PartialEq + Default + 'static>(args: &mut GemmArgs<'_, T>) -> Result<(), Error> {
    args.validate()?;
    let (m, n, k) = (args.m, args.n, args.k);
    if m == 0 || n == 0 {
        return Ok(());
    }
    // Row-major (i, j) lives at i*ld + j: row stride ld, column stride 1; a transpose swaps the two.
    let (a_rs, a_cs) = strides(args.trans_a, args.lda);
    let (b_rs, b_cs) = strides(args.trans_b, args.ldb);
    let read_dst = args.beta != T::default();
    // gemm crate: dst := alpha_g * dst + beta_g * lhs * rhs. BLAS alpha/beta map crosswise.
    // SAFETY: `validate` proved every (row, col) index of A, B and C lies inside its slice, and
    // `args.c` is a unique borrow so the destination aliases neither input.
    unsafe {
        gemm::gemm(
            m,
            n,
            k,
            args.c.as_mut_ptr(),
            1,
            args.ldc as isize,
            read_dst,
            args.a.as_ptr(),
            a_cs,
            a_rs,
            args.b.as_ptr(),
            b_cs,
            b_rs,
            args.beta,
            args.alpha,
            false,
            false,
            false,
            gemm::Parallelism::None,
        );
    }
    Ok(())
}

fn strides(t: Trans, ld: usize) -> (isize, isize) {
    match t {
        Trans::No => (ld as isize, 1),
        Trans::Yes => (1, ld as isize),
    }
}
