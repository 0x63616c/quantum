//! Portable metadata and ordered registration for Voxelite's built-in plugins.
//!
//! Contributions contain data only. Hosts bind command IDs to actions and
//! interpret icon, menu, shortcut and context strings without a GUI dependency
//! here. This crate does not load third-party code, WASM or scripts.
mod id;
mod registry;
mod spec;

pub use id::{InvalidPluginId, PluginId};
pub use registry::{RegisteredContribution, Registrar, Registry, RegistryError};
pub use spec::{
    CommandSpec, Contribution, FormatSpec, Manifest, SettingKind, SettingSpec, SettingValue,
};

/// A statically linked plugin that declares metadata and host contributions.
pub trait Plugin {
    /// Describe this plugin and its enablement policy.
    fn manifest(&self) -> Manifest;
    /// Contribute declarations; hosts bind implementations later by ID.
    fn register(&self, r: &mut Registrar);
}

#[cfg(test)]
mod tests;
