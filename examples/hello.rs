//! Smallest useful cpufish program: multiply two 2x2 matrices with the always-available `portable` provider.
//! Run with `cargo run --example hello`; CI runs it and checks the output shown in the README.

use cpufish::{GemmArgs, Registry, Trans};

fn main() -> Result<(), cpufish::Error> {
    let registry = Registry::with_portable();
    let provider = registry
        .available()
        .into_iter()
        .next()
        .expect("portable is always available");
    let (a, b) = ([1.0_f32, 2.0, 3.0, 4.0], [5.0_f32, 6.0, 7.0, 8.0]); // 2x2 row-major
    let mut c = [0.0_f32; 4];
    provider.gemm_f32(&mut GemmArgs {
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
    })?;
    println!("{} computed {c:?}", provider.id());
    Ok(())
}
