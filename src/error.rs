// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::types::{Dtype, Op};

/// Why a provider could not be used on this machine.
#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    /// The vendor library could not be loaded or initialised.
    #[error("library unavailable: {0}")]
    Unavailable(String),
    /// The library loaded but its known-answer check failed.
    #[error("known-answer check failed: {0}")]
    WrongAnswer(String),
}

/// Errors from running an operation through a provider.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The provider does not implement this operation for this dtype.
    #[error("provider `{provider}` does not support {op:?} for {dtype:?}")]
    Unsupported {
        /// The provider id.
        provider: &'static str,
        /// The operation requested.
        op: Op,
        /// The element type requested.
        dtype: Dtype,
    },
    /// The arguments are inconsistent (leading dimension too small, buffer too short, ...).
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    /// The vendor library reported a failure.
    #[error("{provider}: {message}")]
    Backend {
        /// The provider id.
        provider: &'static str,
        /// The vendor's message.
        message: String,
    },
    /// No provider with this id is installed.
    #[error("no provider `{0}` installed")]
    NoSuchProvider(String),
}
