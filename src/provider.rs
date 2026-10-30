use crate::{Capability, Config, Dtype, Error, GemmArgs, Op, ProbeError};

/// A CPU math library behind a uniform interface.
///
/// A provider lists what it supports in [`capabilities`](Provider::capabilities) and overrides only those
/// typed methods; the defaults return [`Error::Unsupported`].
pub trait Provider: Send + Sync {
    /// Stable short id: `"portable"`, `"aocl"`, `"onemkl"`, ...
    fn id(&self) -> &'static str;

    /// The library's own version string, if it reports one.
    fn version(&self) -> Option<String>;

    /// Load the library and run a known-answer check. Must not change global state.
    fn probe(&self) -> Result<(), ProbeError>;

    /// What this provider claims to implement.
    fn capabilities(&self) -> &[Capability];

    /// The thread count and instruction set actually in force.
    fn effective_config(&self) -> Config;

    /// Single-precision gemm.
    fn gemm_f32(&self, _args: &mut GemmArgs<'_, f32>) -> Result<(), Error> {
        Err(Error::Unsupported {
            provider: self.id(),
            op: Op::Gemm,
            dtype: Dtype::F32,
        })
    }

    /// Double-precision gemm.
    fn gemm_f64(&self, _args: &mut GemmArgs<'_, f64>) -> Result<(), Error> {
        Err(Error::Unsupported {
            provider: self.id(),
            op: Op::Gemm,
            dtype: Dtype::F64,
        })
    }
}
