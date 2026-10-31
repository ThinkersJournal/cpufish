// SPDX-License-Identifier: MIT OR Apache-2.0
//! CPUFish: a vendor-neutral CPU math layer with per-vendor providers selected at run time.
//!
//! This is the core crate (ROADMAP S1): the [`Provider`] trait, the [`Registry`], the always-available
//! [`portable`] provider and a shared [`conformance`] suite every provider must pass. Vendor providers
//! (AOCL, oneMKL, OpenBLAS, ...) live in their own crates. See `docs/PROVIDER-DESIGN.md`.

pub mod conformance;
pub mod portable;

mod error;
mod provider;
mod registry;
mod types;

pub use error::{Error, ProbeError};
pub use provider::Provider;
pub use registry::Registry;
pub use types::{Capability, Config, Dtype, GemmArgs, Op, Trans};
