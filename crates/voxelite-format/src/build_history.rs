use crate::*;
use std::collections::BTreeSet;

/// A bounded, portable recipe that builds a project from an empty default scene.
/// This is build provenance, not a serialization of a host's undo/redo session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildHistory {
    /// Recipe schema, currently 1.
    pub version: u16,
    /// Identity of the initial empty Base layer.
    pub base_layer: Uuid,
    /// Ordered, nonempty edits. Each must produce exactly one history entry.
    pub steps: Vec<BuildStep>,
}

/// One actual core command in a reproducible build.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildStep {
    /// Stable operation identity.
    pub id: Uuid,
    /// Human-readable action label.
    pub label: String,
    /// Portable author role, without local client identities.
    pub origin: Origin,
    /// Logical milliseconds since build start, never a wall-clock timestamp.
    pub elapsed_ms: u32,
    /// Typed command, validated and applied by the core engine on replay.
    pub command: Command,
}

impl BuildHistory {
    pub(crate) fn replay(&self, document: Uuid, limits: Limits) -> FormatResult<Engine> {
        if self.version != 1 || self.steps.is_empty() || self.steps.len() > 64 {
            return Err(FormatError::Invalid(
                "build history requires version 1 and 1..=64 steps".into(),
            ));
        }
        let mut engine = Engine::with_limits(Document::new(document, self.base_layer), limits)?;
        let mut elapsed = None;
        let mut work = 0;
        let mut ids = BTreeSet::new();
        for step in &self.steps {
            if !ids.insert(step.id) {
                return Err(FormatError::Invalid("duplicate build step ID".into()));
            }
            if elapsed.is_some_and(|previous| step.elapsed_ms <= previous) {
                return Err(FormatError::Invalid("build times must increase".into()));
            }
            elapsed = Some(step.elapsed_ms);
            let receipt = engine.execute(Transaction {
                expected_revision: engine.revision(),
                id: step.id,
                actor: Actor::new(step.origin),
                label: step.label.clone(),
                command: step.command.clone(),
            })?;
            work += receipt.work_units;
            if receipt.entry.is_none() || work > 4 * MAX_WORK {
                return Err(FormatError::Invalid(
                    "build history contains a no-op or exceeds replay budgets".into(),
                ));
            }
        }
        engine.mark_saved();
        Ok(engine)
    }
}

impl Project {
    /// Attach a recipe only after real command replay reproduces the entire document.
    pub fn set_build_history(&mut self, history: BuildHistory) -> FormatResult<()> {
        let engine = history.replay(self.document.id, Limits::default())?;
        if !engine.initial_retained() {
            return Err(FormatError::Invalid(
                "build exceeds the default history budget".into(),
            ));
        }
        if engine.document() != &self.document {
            return Err(FormatError::Invalid(
                "build history does not reproduce the project".into(),
            ));
        }
        self.built_document = Some(self.document.clone());
        self.build_history = Some(history);
        Ok(())
    }

    /// Recipe for the current document. Editing content invalidates the original recipe;
    /// it is then omitted on save rather than reopening an unrelated undo timeline.
    pub fn build_history(&self) -> Option<&BuildHistory> {
        self.build_history
            .as_ref()
            .filter(|_| self.built_document.as_ref() == Some(&self.document))
    }

    /// Open saved content with its validated build timeline, or an empty runtime history.
    /// Replayed edits are marked saved and respect the host's history/work budgets.
    /// Smaller host budgets may evict early entries, as with ordinary edits.
    pub fn open_engine(&self, limits: Limits) -> FormatResult<Engine> {
        match self.build_history() {
            Some(history) => history.replay(self.document.id, limits),
            None => Ok(Engine::with_limits(self.document.clone(), limits)?),
        }
    }
}
