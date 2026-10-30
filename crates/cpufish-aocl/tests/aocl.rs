//! These need AOCL installed and findable at run time. They are UNVERIFIED in CI until a self-hosted
//! runner exists for this repo (see ROADMAP); run locally with `cargo test -p cpufish-aocl`.

use cpufish::{Provider, conformance};
use cpufish_aocl::Aocl;

#[test]
fn aocl_passes_conformance() {
    conformance::check_provider(&Aocl::new()).unwrap();
}

#[test]
fn aocl_probe_succeeds() {
    Aocl::new().probe().unwrap();
}

#[test]
fn aocl_reports_version_threads_and_isa() {
    let p = Aocl::new();
    let v = p.version().expect("BLIS reports a version");
    assert!(!v.is_empty());
    let cfg = p.effective_config();
    assert!(cfg.threads.is_some_and(|t| t >= 1), "{cfg:?}");
    assert!(cfg.isa.as_ref().is_some_and(|s| !s.is_empty()), "{cfg:?}");
    eprintln!("aocl version={v} config={cfg:?}");
}
