use std::collections::BTreeSet;

use crate::{
    CommandContribution, CommandTransformContribution, Contribution, DataSectionContribution,
    FormatContribution, PanelContribution, RenderPassContribution, SettingContribution,
    SettingKind, SettingValue, ToolContribution,
};

/// Collects declarations during [`crate::Plugin::contribute`].
///
/// Use a fresh registrar for each plugin. Validation happens on [`Self::finish`],
/// so an invalid declaration cannot be silently dropped during registration.
/// Cross-plugin conflicts, enablement and lifecycle are the host's concern.
#[derive(Debug, Default)]
pub struct Registrar {
    contributions: Vec<Contribution>,
}

impl Registrar {
    /// Append a declaration, preserving order across contribution kinds.
    pub fn contribute(&mut self, contribution: Contribution) {
        self.contributions.push(contribution);
    }

    /// Append a command declaration.
    pub fn command(&mut self, contribution: CommandContribution) {
        self.contribute(Contribution::Command(contribution));
    }

    /// Append an interactive tool declaration.
    pub fn tool(&mut self, contribution: ToolContribution) {
        self.contribute(Contribution::Tool(contribution));
    }

    /// Append a pending command transform declaration.
    pub fn command_transform(&mut self, contribution: CommandTransformContribution) {
        self.contribute(Contribution::CommandTransform(contribution));
    }

    /// Append a panel declaration.
    pub fn panel(&mut self, contribution: PanelContribution) {
        self.contribute(Contribution::Panel(contribution));
    }

    /// Append a file format declaration.
    pub fn format(&mut self, contribution: FormatContribution) {
        self.contribute(Contribution::Format(contribution));
    }

    /// Append a render pass declaration.
    pub fn render_pass(&mut self, contribution: RenderPassContribution) {
        self.contribute(Contribution::RenderPass(contribution));
    }

    /// Append a setting declaration.
    pub fn setting(&mut self, contribution: SettingContribution) {
        self.contribute(Contribution::Setting(contribution));
    }

    /// Append a `.voxl` data section declaration.
    pub fn data_section(&mut self, contribution: DataSectionContribution) {
        self.contribute(Contribution::DataSection(contribution));
    }

    /// Validate this plugin's declarations without exposing a partial result.
    pub fn finish(self) -> Result<Contributions, ValidationError> {
        validate_contributions(&self.contributions)?;
        Ok(Contributions(self.contributions))
    }
}

/// Validated declarations for one plugin in registration order.
///
/// Built through [`Registrar::finish`]. This collection is not a registry: it
/// knows nothing about other plugins, enabled sets or implementation bindings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contributions(Vec<Contribution>);

impl Contributions {
    /// Declarations in registration order across all kinds.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Contribution> {
        self.0.iter()
    }

    /// Number of declarations.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the plugin contributed no declarations.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A plugin's declarations failed local validation.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ValidationError {
    /// Two declarations in this plugin share an identifier, including across kinds.
    #[error("duplicate contribution id {id:?}")]
    DuplicateContribution {
        /// Repeated contribution ID, setting key or section name.
        id: String,
    },
    /// A declaration has an invalid identifier or setting schema.
    #[error("invalid contribution {id:?}: {reason}")]
    InvalidContribution {
        /// Invalid contribution ID, setting key or section name.
        id: String,
        /// Explanation of the invalid declaration.
        reason: String,
    },
}

/// Check identifier syntax, duplicate IDs and setting defaults within one plugin.
///
/// Keybinding conflicts, command JSON Schema interpretation, references to host
/// resources and cross-plugin validation are deliberately left to the host.
pub fn validate_contributions(contributions: &[Contribution]) -> Result<(), ValidationError> {
    let mut ids = BTreeSet::new();
    for contribution in contributions {
        let id = contribution.id();
        if let Some(reason) = invalid_reason(contribution) {
            return Err(ValidationError::InvalidContribution {
                id: id.to_owned(),
                reason: reason.into(),
            });
        }
        if !ids.insert(id) {
            return Err(ValidationError::DuplicateContribution { id: id.to_owned() });
        }
    }
    Ok(())
}

fn invalid_reason(contribution: &Contribution) -> Option<&'static str> {
    let id = contribution.id();
    if !id.split('.').all(|segment| {
        segment
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_lowercase)
            && segment.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
            })
    }) {
        return Some("expected dot-separated segments matching [a-z][a-z0-9_-]*");
    }
    if !matches!(contribution, Contribution::DataSection(_)) && !id.contains('.') {
        return Some("expected a namespaced ID with at least two dot-separated segments");
    }
    if let Contribution::DataSection(section) = contribution
        && (section.schema_version == 0 || !quantum_core::valid_section_name(id))
    {
        return Some("data section requires a bounded section name and positive schema_version");
    }
    if let Contribution::Setting(setting) = contribution {
        match (&setting.kind, &setting.default) {
            (SettingKind::Bool, SettingValue::Bool(_)) => {}
            (SettingKind::Choice(choices), SettingValue::Choice(default)) => {
                let mut seen = BTreeSet::new();
                if choices.is_empty()
                    || choices
                        .iter()
                        .any(|choice| choice.is_empty() || !seen.insert(choice))
                {
                    return Some("choices must be nonempty and contain distinct, nonempty values");
                }
                if !choices.contains(default) {
                    return Some("default must be one of the declared choices");
                }
            }
            (SettingKind::Number, SettingValue::Number(default)) => {
                if !default.is_finite() {
                    return Some("numeric default must be finite");
                }
            }
            _ => return Some("default value must match the setting kind"),
        }
    }
    None
}
