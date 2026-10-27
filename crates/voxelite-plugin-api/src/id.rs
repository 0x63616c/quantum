use std::{fmt, str::FromStr};

/// A stable reverse-DNS plugin namespace, such as `dev.voxelite.tools`.
///
/// IDs have at least two dot-separated labels and at most 253 ASCII bytes.
/// Each label has 1–63 bytes, starts with a lowercase letter, contains only
/// lowercase letters, digits or hyphens, and ends with a letter or digit.
/// Input is validated as written, without trimming or case folding. Validation
/// checks syntax only, not domain ownership; an ID must remain stable on rename.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginId(String);

/// A plugin ID does not follow the lowercase reverse-DNS namespace grammar.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "invalid plugin id {id:?}: expected a lowercase reverse-DNS namespace (2+ labels, 1–63 bytes each, 253 bytes total)"
)]
pub struct InvalidPluginId {
    /// The rejected input.
    pub id: String,
}

impl PluginId {
    /// Validate a plugin namespace.
    pub fn new(id: impl Into<String>) -> Result<Self, InvalidPluginId> {
        let id = id.into();
        if !is_namespaced(&id) {
            return Err(InvalidPluginId { id });
        }
        Ok(Self(id))
    }

    /// Borrow the validated namespace.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for PluginId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for PluginId {
    type Err = InvalidPluginId;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Self::new(id)
    }
}

impl TryFrom<String> for PluginId {
    type Error = InvalidPluginId;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::new(id)
    }
}

pub(crate) fn is_namespaced(id: &str) -> bool {
    voxelite_core::valid_plugin_namespace(id)
}
