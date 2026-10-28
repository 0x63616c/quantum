//! Capability-scoped access to durable plugin data, without document or I/O access.
pub use quantum_core::DataSection;

/// A plugin rejected section data or attempted an undeclared access.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct DataError(pub String);
impl From<crate::CoreError> for DataError {
    fn from(error: crate::CoreError) -> Self {
        Self(error.to_string())
    }
}
/// Result of section access, validation or migration.
pub type DataResult<T> = Result<T, DataError>;

/// Read/write access only to this enabled plugin's declared sections.
/// Writes are staged; errors or panics discard the entire operation.
pub trait DataCx {
    /// Read a declared section, or None when it has never been stored.
    fn read(&self, section_name: &str) -> DataResult<Option<&DataSection>>;
    /// Replace a declared section at its current schema version; None deletes it.
    fn write(&mut self, section_name: &str, section: Option<DataSection>) -> DataResult<()>;
}

/// One deterministic forward migration. The host validates the chain and result.
#[derive(Clone, Copy)]
pub struct DataMigration {
    /// Input schema version (one outgoing edge per version).
    pub from_version: u32,
    /// Output schema version, strictly greater than the input.
    pub to_version: u32,
    /// Pure conversion of one section. No filesystem, engine or other plugin access.
    pub migrate: fn(&DataSection) -> DataResult<DataSection>,
}
