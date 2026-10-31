// SPDX-License-Identifier: MIT OR Apache-2.0
use std::sync::Arc;

use cpufish::{
    Config, Dtype, Error, GemmArgs, Op, Provider, Registry, Trans, conformance, portable::Portable,
};

#[test]
fn portable_passes_conformance() {
    conformance::check_provider(&Portable::new()).unwrap();
}

#[test]
fn registry_installs_and_looks_up() {
    let r = Registry::with_portable();
    assert_eq!(r.ids(), ["portable"]);
    assert_eq!(r.get("portable").unwrap().id(), "portable");
    assert!(matches!(r.get("aocl"), Err(Error::NoSuchProvider(_))));
    assert_eq!(r.available().len(), 1);
}

#[test]
fn install_replaces_same_id() {
    let mut r = Registry::with_portable();
    r.install(Arc::new(Portable::new()));
    assert_eq!(r.ids().len(), 1);
}

#[test]
fn portable_reports_its_claims_and_config() {
    let p = Portable::new();
    assert!(
        p.capabilities()
            .iter()
            .any(|c| c.op == Op::Gemm && c.dtype == Dtype::F32)
    );
    assert_eq!(p.effective_config().threads, Some(1));
    assert_ne!(p.effective_config(), Config::default());
}

/// A provider that claims nothing must report `Unsupported`, never wrong numbers.
struct Nothing;

impl Provider for Nothing {
    fn id(&self) -> &'static str {
        "nothing"
    }
    fn version(&self) -> Option<String> {
        None
    }
    fn probe(&self) -> Result<(), cpufish::ProbeError> {
        Ok(())
    }
    fn capabilities(&self) -> &[cpufish::Capability] {
        &[]
    }
    fn effective_config(&self) -> Config {
        Config::default()
    }
}

#[test]
fn unclaimed_operations_are_unsupported() {
    conformance::check_provider(&Nothing).unwrap();
    let one = [1.0_f32];
    let mut c = [0.0_f32];
    let mut args = GemmArgs {
        trans_a: Trans::No,
        trans_b: Trans::No,
        m: 1,
        n: 1,
        k: 1,
        alpha: 1.0,
        a: &one,
        lda: 1,
        b: &one,
        ldb: 1,
        beta: 0.0,
        c: &mut c,
        ldc: 1,
    };
    assert!(matches!(
        Nothing.gemm_f32(&mut args),
        Err(Error::Unsupported { .. })
    ));
}

/// Negative control: a provider that claims gemm but computes garbage must fail the suite.
struct Liar;

impl Provider for Liar {
    fn id(&self) -> &'static str {
        "liar"
    }
    fn version(&self) -> Option<String> {
        None
    }
    fn probe(&self) -> Result<(), cpufish::ProbeError> {
        Ok(())
    }
    fn capabilities(&self) -> &[cpufish::Capability] {
        const C: [cpufish::Capability; 1] = [cpufish::Capability {
            op: Op::Gemm,
            dtype: Dtype::F32,
            notes: "",
        }];
        &C
    }
    fn effective_config(&self) -> Config {
        Config::default()
    }
    fn gemm_f32(&self, args: &mut GemmArgs<'_, f32>) -> Result<(), Error> {
        args.validate()?;
        Ok(())
    }
}

#[test]
fn conformance_catches_a_wrong_provider() {
    assert!(conformance::check_provider(&Liar).is_err());
}

#[test]
fn set_threads_portable_accepts_only_one_and_default_refuses() {
    assert!(Portable::new().set_threads(1).is_ok());
    assert!(matches!(
        Portable::new().set_threads(4),
        Err(Error::InvalidArgument(_))
    ));
    assert!(matches!(Liar.set_threads(1), Err(Error::Backend { .. })));
}
