// SPDX-License-Identifier: MIT OR Apache-2.0
use std::sync::Arc;

use crate::{Error, Provider};

/// The set of installed providers. Installing a provider does not load its library; call
/// [`Registry::available`] (or `probe` on one provider) for that.
#[derive(Default)]
pub struct Registry {
    providers: Vec<Arc<dyn Provider>>,
}

impl Registry {
    /// An empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// A registry holding the always-available [`portable`](crate::portable) provider.
    pub fn with_portable() -> Self {
        let mut r = Self::new();
        r.install(Arc::new(crate::portable::Portable::new()));
        r
    }

    /// Install a provider, replacing any installed provider with the same id.
    pub fn install(&mut self, provider: Arc<dyn Provider>) {
        self.providers.retain(|p| p.id() != provider.id());
        self.providers.push(provider);
    }

    /// Ids of all installed providers, in installation order.
    pub fn ids(&self) -> Vec<&'static str> {
        self.providers.iter().map(|p| p.id()).collect()
    }

    /// Look up a provider by id.
    pub fn get(&self, id: &str) -> Result<Arc<dyn Provider>, Error> {
        self.providers
            .iter()
            .find(|p| p.id() == id)
            .cloned()
            .ok_or_else(|| Error::NoSuchProvider(id.to_owned()))
    }

    /// Providers whose `probe` succeeds on this machine, in installation order.
    pub fn available(&self) -> Vec<Arc<dyn Provider>> {
        self.providers
            .iter()
            .filter(|p| p.probe().is_ok())
            .cloned()
            .collect()
    }
}
