//! These need oneMKL installed and findable at run time. UNVERIFIED in CI until a self-hosted runner exists
//! for this repo (see ROADMAP); run locally with `cargo test -p cpufish-onemkl`.

use cpufish::{Provider, conformance};
use cpufish_onemkl::OneMkl;

#[test]
fn onemkl_passes_conformance() {
    conformance::check_provider(&OneMkl::new()).unwrap();
}

#[test]
fn onemkl_probe_succeeds() {
    OneMkl::new().probe().unwrap();
}

#[test]
fn onemkl_reports_version_threads_and_isa() {
    let p = OneMkl::new();
    let v = p.version().expect("oneMKL reports a version");
    assert!(!v.is_empty());
    let cfg = p.effective_config();
    assert!(cfg.threads.is_some_and(|t| t >= 1), "{cfg:?}");
    assert!(cfg.isa.as_ref().is_some_and(|s| !s.is_empty()), "{cfg:?}");
    eprintln!("onemkl version={v} config={cfg:?}");
}
