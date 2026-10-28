//! Read-side aliases for persisted built-in enablement and setting identifiers.
use std::borrow::Cow;

pub(crate) fn canonical_id(id: &str) -> Cow<'_, str> {
    match id.strip_prefix("dev.voxelite.") {
        Some(suffix) => Cow::Owned(format!("dev.quantum.{suffix}")),
        None => Cow::Borrowed(id),
    }
}
