//! The shared conformance suite: every provider must pass [`check_provider`] before it is trusted.
//!
//! The suite checks a provider's *claims* ([`Provider::capabilities`]) against a naive reference
//! computed in `f64`. It tests correctness within a rounding tolerance only; it says nothing about
//! determinism or ULP bounds, which consumers measure for themselves.

use crate::{Dtype, Error, GemmArgs, Op, Provider, Trans};

trait Elem: Copy + PartialEq + Default + std::fmt::Debug {
    const DTYPE: Dtype;
    const EPS: f64;
    fn from_f64(v: f64) -> Self;
    fn to_f64(self) -> f64;
    fn run(p: &dyn Provider, args: &mut GemmArgs<'_, Self>) -> Result<(), Error>;
}

impl Elem for f32 {
    const DTYPE: Dtype = Dtype::F32;
    const EPS: f64 = f32::EPSILON as f64;
    fn from_f64(v: f64) -> Self {
        v as f32
    }
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn run(p: &dyn Provider, args: &mut GemmArgs<'_, Self>) -> Result<(), Error> {
        p.gemm_f32(args)
    }
}

impl Elem for f64 {
    const DTYPE: Dtype = Dtype::F64;
    const EPS: f64 = f64::EPSILON;
    fn from_f64(v: f64) -> Self {
        v
    }
    fn to_f64(self) -> f64 {
        self
    }
    fn run(p: &dyn Provider, args: &mut GemmArgs<'_, Self>) -> Result<(), Error> {
        p.gemm_f64(args)
    }
}

/// Deterministic pseudo-random values in `[-1, 1)` (SplitMix64), so failures reproduce.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 52) as f64 - 1.0
    }
}

/// Run every check against `provider`. `Err` names the first failing case.
pub fn check_provider(provider: &dyn Provider) -> Result<(), String> {
    provider
        .probe()
        .map_err(|e| format!("{}: probe failed: {e}", provider.id()))?;
    check_dtype::<f32>(provider)?;
    check_dtype::<f64>(provider)
}

fn claims(p: &dyn Provider, dtype: Dtype) -> bool {
    p.capabilities()
        .iter()
        .any(|c| c.op == Op::Gemm && c.dtype == dtype)
}

fn check_dtype<T: Elem>(p: &dyn Provider) -> Result<(), String> {
    let id = p.id();
    if !claims(p, T::DTYPE) {
        // An unclaimed operation must fail loudly rather than return wrong numbers.
        let mut c = [T::default(); 1];
        let one = [T::from_f64(1.0)];
        let mut args = args_for(
            Trans::No,
            Trans::No,
            1,
            1,
            1,
            one[0],
            &one,
            1,
            &one,
            1,
            T::default(),
            &mut c,
            1,
        );
        return match T::run(p, &mut args) {
            Err(Error::Unsupported { .. }) => Ok(()),
            other => Err(format!(
                "{id}: {:?} gemm not claimed but returned {other:?}",
                T::DTYPE
            )),
        };
    }
    let mut rng = Rng(0x00C0_FFEE);
    let shapes = [
        (1, 1, 1),
        (2, 2, 2),
        (3, 5, 7),
        (17, 13, 9),
        (33, 31, 29),
        (0, 4, 4),
        (4, 0, 4),
        (4, 4, 0),
    ];
    let scales = [(1.0, 0.0), (2.0, 0.5), (0.0, 1.0), (-1.5, -2.0)];
    for &(m, n, k) in &shapes {
        for ta in [Trans::No, Trans::Yes] {
            for tb in [Trans::No, Trans::Yes] {
                for &(alpha, beta) in &scales {
                    for pad in [0, 3] {
                        case::<T>(p, &mut rng, (m, n, k), (ta, tb), (alpha, beta), pad)
                            .map_err(|e| format!("{id} {:?}: m={m} n={n} k={k} {ta:?}/{tb:?} alpha={alpha} beta={beta} pad={pad}: {e}", T::DTYPE))?;
                    }
                }
            }
        }
    }
    nan_c_is_not_read::<T>(p).map_err(|e| format!("{id} {:?}: {e}", T::DTYPE))?;
    rejects_bad_arguments::<T>(p).map_err(|e| format!("{id} {:?}: {e}", T::DTYPE))
}

#[allow(clippy::too_many_arguments)]
fn args_for<'a, T>(
    ta: Trans,
    tb: Trans,
    m: usize,
    n: usize,
    k: usize,
    alpha: T,
    a: &'a [T],
    lda: usize,
    b: &'a [T],
    ldb: usize,
    beta: T,
    c: &'a mut [T],
    ldc: usize,
) -> GemmArgs<'a, T> {
    GemmArgs {
        trans_a: ta,
        trans_b: tb,
        m,
        n,
        k,
        alpha,
        a,
        lda,
        b,
        ldb,
        beta,
        c,
        ldc,
    }
}

/// Stored shape (rows, cols) of an operand whose logical shape is `rows x cols`.
fn stored(t: Trans, rows: usize, cols: usize) -> (usize, usize) {
    match t {
        Trans::No => (rows, cols),
        Trans::Yes => (cols, rows),
    }
}

fn fill<T: Elem>(rng: &mut Rng, rows: usize, cols: usize, ld: usize) -> Vec<T> {
    // Padding holds a sentinel so a provider that reads or writes outside the matrix is caught.
    let mut v =
        vec![T::from_f64(777.0); rows.saturating_sub(1) * ld + if rows > 0 { cols } else { 0 }];
    for r in 0..rows {
        for c in 0..cols {
            v[r * ld + c] = T::from_f64(rng.next());
        }
    }
    v
}

fn case<T: Elem>(
    p: &dyn Provider,
    rng: &mut Rng,
    (m, n, k): (usize, usize, usize),
    (ta, tb): (Trans, Trans),
    (alpha, beta): (f64, f64),
    pad: usize,
) -> Result<(), String> {
    let (ar, ac) = stored(ta, m, k);
    let (br, bc) = stored(tb, k, n);
    let (lda, ldb, ldc) = (ac.max(1) + pad, bc.max(1) + pad, n.max(1) + pad);
    let a = fill::<T>(rng, ar, ac, lda);
    let b = fill::<T>(rng, br, bc, ldb);
    let mut c = fill::<T>(rng, m, n, ldc);
    let c0 = c.clone();

    let at = |i: usize, l: usize| {
        a[match ta {
            Trans::No => i * lda + l,
            Trans::Yes => l * lda + i,
        }]
        .to_f64()
    };
    let bt = |l: usize, j: usize| {
        b[match tb {
            Trans::No => l * ldb + j,
            Trans::Yes => j * ldb + l,
        }]
        .to_f64()
    };
    let mut want = c0.clone();
    let mut bound = 0.0_f64;
    for i in 0..m {
        for j in 0..n {
            let mut dot = 0.0;
            let mut mag = 0.0;
            for l in 0..k {
                dot += at(i, l) * bt(l, j);
                mag += (at(i, l) * bt(l, j)).abs();
            }
            let old = if beta == 0.0 {
                0.0
            } else {
                beta * c0[i * ldc + j].to_f64()
            };
            want[i * ldc + j] = T::from_f64(alpha * dot + old);
            bound = bound.max(alpha.abs() * mag + (beta * c0[i * ldc + j].to_f64()).abs());
        }
    }
    let tol = T::EPS * (k as f64 + 4.0) * 4.0 * bound.max(1.0);

    let mut args = args_for(
        ta,
        tb,
        m,
        n,
        k,
        T::from_f64(alpha),
        &a,
        lda,
        &b,
        ldb,
        T::from_f64(beta),
        &mut c,
        ldc,
    );
    T::run(p, &mut args).map_err(|e| e.to_string())?;
    for idx in 0..c.len() {
        let in_matrix = m > 0 && n > 0 && idx % ldc < n;
        let (got, exp) = (c[idx].to_f64(), want[idx].to_f64());
        if in_matrix {
            if (got - exp).abs() > tol {
                return Err(format!(
                    "element {idx}: got {got}, want {exp} (tol {tol:e})"
                ));
            }
        } else if c[idx] != c0[idx] {
            return Err(format!("padding element {idx} was modified"));
        }
    }
    Ok(())
}

fn nan_c_is_not_read<T: Elem>(p: &dyn Provider) -> Result<(), String> {
    let one = [T::from_f64(1.0)];
    let mut c = [T::from_f64(f64::NAN); 1];
    let mut args = args_for(
        Trans::No,
        Trans::No,
        1,
        1,
        1,
        one[0],
        &one,
        1,
        &one,
        1,
        T::default(),
        &mut c,
        1,
    );
    T::run(p, &mut args).map_err(|e| e.to_string())?;
    if c[0].to_f64() != 1.0 {
        return Err(format!("beta = 0 must not read C: got {:?}", c[0]));
    }
    Ok(())
}

fn rejects_bad_arguments<T: Elem>(p: &dyn Provider) -> Result<(), String> {
    let one = T::from_f64(1.0);
    let v = [one; 4];
    let mut c = [T::default(); 4];
    // Leading dimension smaller than the column count.
    let mut args = args_for(
        Trans::No,
        Trans::No,
        2,
        2,
        2,
        one,
        &v,
        1,
        &v,
        2,
        T::default(),
        &mut c,
        2,
    );
    if T::run(p, &mut args).is_ok() {
        return Err("lda < columns was accepted".to_owned());
    }
    // Buffer shorter than the matrix.
    let short = [one; 3];
    let mut args = args_for(
        Trans::No,
        Trans::No,
        2,
        2,
        2,
        one,
        &short,
        2,
        &v,
        2,
        T::default(),
        &mut c,
        2,
    );
    if T::run(p, &mut args).is_ok() {
        return Err("short A buffer was accepted".to_owned());
    }
    Ok(())
}
