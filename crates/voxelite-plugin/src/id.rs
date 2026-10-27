use std::{fmt, str::FromStr};

/// A lowercase, dotted plugin namespace, such as `voxelite.tools`.
///
/// IDs have at least two dot-separated segments. Each segment starts with an
/// ASCII lowercase letter and continues with lowercase letters, digits, `_` or
/// `-`. Input is validated as written, without trimming or case folding.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginId(String);

/// A plugin ID does not follow the lowercase dotted namespace grammar.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "invalid plugin id {id:?}: expected at least two dot-separated segments matching [a-z][a-z0-9_-]*"
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
    id.contains('.')
        && id.split('.').all(|segment| {
            segment
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_lowercase)
                && segment.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'_' | b'-')
                })
        })
}
