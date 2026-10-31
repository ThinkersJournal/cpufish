// SPDX-License-Identifier: MIT OR Apache-2.0
//! Compare installed CPUFish providers on gemm shapes. Run: `cargo run --release -p cpufish-bench --features aocl,onemkl`.
//!
//! Shapes follow how Fuel calls gemm (docs/S2-AOCL-DESIGN.md, Fuel lane answers 2026-10-30): f32, NoTrans/NoTrans,
//! alpha = 1, beta = 0, one gemm per batch slice (no batched gemm until measured). Every provider's output is
//! checked against `portable` first, so a fast wrong answer is reported as such, not as a result.
//!
//! Usage: `cpufish-bench [--quick] [--reps N] [--threads N]`. Without `--threads` providers run at library defaults (as Fuel does). Output is a Markdown table on stdout; configuration goes first.

use std::sync::Arc;
use std::time::Instant;

use cpufish::{GemmArgs, Provider, Registry, Trans};

/// (label, m, n, k). Square sizes, then transformer-style projections with a token count `m`.
const SHAPES: &[(&str, usize, usize, usize)] = &[
    ("square 128", 128, 128, 128),
    ("square 512", 512, 512, 512),
    ("square 1024", 1024, 1024, 1024),
    ("decode 1x4096x4096", 1, 4096, 4096),
    ("prefill 32x4096x4096", 32, 4096, 4096),
    ("prefill 256x4096x4096", 256, 4096, 4096),
    ("ffn 256x11008x4096", 256, 11008, 4096),
    ("attn-slice 256x256x128", 256, 256, 128),
];
const QUICK: &[usize] = &[0, 1, 3, 7];

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
    }
}

#[allow(
    unused_mut,
    reason = "vendors are installed only when their feature is on"
)]
fn registry() -> Registry {
    let mut r = Registry::with_portable();
    #[cfg(feature = "aocl")]
    r.install(Arc::new(cpufish_aocl::Aocl::new()));
    #[cfg(feature = "onemkl")]
    r.install(Arc::new(cpufish_onemkl::OneMkl::new()));
    r
}

fn run(p: &Arc<dyn Provider>, m: usize, n: usize, k: usize, a: &[f32], b: &[f32], c: &mut [f32]) {
    let mut args = GemmArgs {
        trans_a: Trans::No,
        trans_b: Trans::No,
        m,
        n,
        k,
        alpha: 1.0,
        a,
        lda: k,
        b,
        ldb: n,
        beta: 0.0,
        c,
        ldc: n,
    };
    p.gemm_f32(&mut args).expect("gemm failed");
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|x, y| x.total_cmp(y));
    v[v.len() / 2]
}

fn main() {
    let mut quick = false;
    let mut reps = 5usize;
    let mut threads: Option<usize> = None;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--quick" => quick = true,
            "--reps" => reps = it.next().and_then(|v| v.parse().ok()).expect("--reps N"),
            "--threads" => {
                threads = Some(it.next().and_then(|v| v.parse().ok()).expect("--threads N"));
            }
            other => panic!("unknown argument {other}"),
        }
    }
    let reg = registry();
    let providers: Vec<Arc<dyn Provider>> = reg
        .ids()
        .into_iter()
        .filter_map(|id| {
            let p = reg.get(id).ok()?;
            match p.probe() {
                Ok(()) => Some(p),
                Err(e) => {
                    eprintln!("skipping {id}: {e}");
                    None
                }
            }
        })
        .collect();
    // Thread settings are process-global in BLIS and MKL: apply once, before any gemm, then report what is in force.
    if let Some(n) = threads {
        for p in &providers {
            if let Err(e) = p.set_threads(n) {
                eprintln!("{}: could not set {n} threads: {e}", p.id());
            }
        }
    }
    println!("## Providers\n");
    match threads {
        Some(n) => {
            println!("Requested threads: {n} (see each provider's effective config below).\n")
        }
        None => println!("Requested threads: none (library defaults).\n"),
    }
    for p in &providers {
        println!(
            "- `{}` version={:?} config={:?}",
            p.id(),
            p.version(),
            p.effective_config()
        );
    }
    println!(
        "\nf32, NoTrans/NoTrans, alpha=1, beta=0, median of {reps} runs after 1 warm-up. GFLOP/s = 2mnk / time."
    );
    println!(
        "\n**Timings from this harness are NOT a ranking of providers: the box is shared and loaded, and thread use is unconfirmed.**\n"
    );
    print!("| shape (m x n x k) |");
    for p in &providers {
        print!(" {} GFLOP/s | {} max err vs portable |", p.id(), p.id());
    }
    println!();
    println!("|---|{}", "---|".repeat(providers.len() * 2));

    let mut rng = Rng(0x00C0_FFEE);
    for (i, &(label, m, n, k)) in SHAPES.iter().enumerate() {
        if quick && !QUICK.contains(&i) {
            continue;
        }
        let a: Vec<f32> = (0..m * k).map(|_| rng.next()).collect();
        let b: Vec<f32> = (0..k * n).map(|_| rng.next()).collect();
        let mut reference = vec![0.0f32; m * n];
        let mut c = vec![0.0f32; m * n];
        let mut row = format!("| {label} |");
        for (pi, p) in providers.iter().enumerate() {
            run(p, m, n, k, &a, &b, &mut c); // warm-up, also the correctness sample
            let err = if pi == 0 {
                reference.copy_from_slice(&c);
                0.0
            } else {
                c.iter()
                    .zip(&reference)
                    .map(|(x, y)| (x - y).abs())
                    .fold(0.0f32, f32::max)
            };
            let mut t: Vec<f64> = (0..reps)
                .map(|_| {
                    let s = Instant::now();
                    run(p, m, n, k, &a, &b, &mut c);
                    s.elapsed().as_secs_f64()
                })
                .collect();
            let gflops = 2.0 * (m * n * k) as f64 / median(&mut t) / 1e9;
            row += &format!(" {gflops:.1} | {err:.2e} |");
        }
        println!("{row}");
    }
}
