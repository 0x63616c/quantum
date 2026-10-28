use std::collections::BTreeMap;

use quantum_plugin_api::{
    CommandContribution, CommandTransformContribution, Contribution, Contributions,
    DataSectionContribution, FormatContribution, PanelContribution, PluginId,
    RenderPassContribution, SettingContribution, ToolContribution,
};

use crate::host::{HostedPlugin, PluginState};

/// Identifies one declaration involved in a conflict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContributionOwner {
    /// Declaring plugin.
    pub plugin_id: PluginId,
    /// Contribution ID, setting key or data section name.
    pub contribution_id: String,
}

/// Declarations that cannot be published together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Conflict {
    /// Two declarations share an identifier, including across different kinds.
    DuplicateContributionId {
        /// Shared identifier.
        id: String,
        /// First declaration in registration order.
        first: ContributionOwner,
        /// Later declaration in registration order.
        second: ContributionOwner,
    },
    /// Two commands use the same canonical binding in the same opaque context.
    DuplicateDefaultKeybinding {
        /// Canonical binding string, compared verbatim.
        keybinding: String,
        /// Opaque context, with `None` denoting the global context.
        context: Option<String>,
        /// First command in registration order.
        first: ContributionOwner,
        /// Later command in registration order.
        second: ContributionOwner,
    },
}

/// A borrowed, read-only view of enabled declarations in registration order.
///
/// Obtain a new view from [`crate::Host::registry`] after a lifecycle transition.
/// Both plugin order and declaration order within each plugin are preserved.
pub struct Registry<'a> {
    pub(crate) plugins: &'a [HostedPlugin],
}

impl<'a> Registry<'a> {
    /// Iterate over every enabled declaration and its owning plugin.
    pub fn iter(&self) -> impl Iterator<Item = (&'a PluginId, &'a Contribution)> + use<'a> {
        self.plugins
            .iter()
            .filter(|plugin| plugin.state == PluginState::Enabled)
            .flat_map(|plugin| {
                plugin
                    .contributions
                    .iter()
                    .map(|contribution| (&plugin.manifest.id, contribution))
            })
    }

    /// Look up an enabled declaration by its globally unique ID or setting key.
    /// Data sections are plugin-scoped; use `for_plugin` or `data_sections` instead.
    pub fn get(&self, id: &str) -> Option<(&'a PluginId, &'a Contribution)> {
        self.iter().find(|(_, contribution)| {
            !matches!(contribution, Contribution::DataSection(_)) && contribution.id() == id
        })
    }

    /// Get all declarations of an enabled plugin, including an empty collection.
    pub fn for_plugin(&self, id: &str) -> Option<&'a Contributions> {
        self.plugins
            .iter()
            .find(|plugin| {
                plugin.manifest.id.as_str() == id && plugin.state == PluginState::Enabled
            })
            .map(|plugin| &plugin.contributions)
    }

    /// Enabled command declarations and their owners.
    pub fn commands(
        &self,
    ) -> impl Iterator<Item = (&'a PluginId, &'a CommandContribution)> + use<'a> {
        self.iter()
            .filter_map(|(id, contribution)| match contribution {
                Contribution::Command(spec) => Some((id, spec)),
                _ => None,
            })
    }

    /// Enabled tool declarations and their owners.
    pub fn tools(&self) -> impl Iterator<Item = (&'a PluginId, &'a ToolContribution)> + use<'a> {
        self.iter()
            .filter_map(|(id, contribution)| match contribution {
                Contribution::Tool(spec) => Some((id, spec)),
                _ => None,
            })
    }

    /// Enabled transform declarations in registration order, before execution sorting.
    pub fn command_transforms(
        &self,
    ) -> impl Iterator<Item = (&'a PluginId, &'a CommandTransformContribution)> + use<'a> {
        self.iter()
            .filter_map(|(id, contribution)| match contribution {
                Contribution::CommandTransform(spec) => Some((id, spec)),
                _ => None,
            })
    }

    /// Enabled panel declarations and their owners.
    pub fn panels(&self) -> impl Iterator<Item = (&'a PluginId, &'a PanelContribution)> + use<'a> {
        self.iter()
            .filter_map(|(id, contribution)| match contribution {
                Contribution::Panel(spec) => Some((id, spec)),
                _ => None,
            })
    }

    /// Enabled format declarations and their owners.
    pub fn formats(
        &self,
    ) -> impl Iterator<Item = (&'a PluginId, &'a FormatContribution)> + use<'a> {
        self.iter()
            .filter_map(|(id, contribution)| match contribution {
                Contribution::Format(spec) => Some((id, spec)),
                _ => None,
            })
    }

    /// Enabled render pass declarations and their owners.
    pub fn render_passes(
        &self,
    ) -> impl Iterator<Item = (&'a PluginId, &'a RenderPassContribution)> + use<'a> {
        self.iter()
            .filter_map(|(id, contribution)| match contribution {
                Contribution::RenderPass(spec) => Some((id, spec)),
                _ => None,
            })
    }

    /// Enabled setting declarations and their owners.
    pub fn settings(
        &self,
    ) -> impl Iterator<Item = (&'a PluginId, &'a SettingContribution)> + use<'a> {
        self.iter()
            .filter_map(|(id, contribution)| match contribution {
                Contribution::Setting(spec) => Some((id, spec)),
                _ => None,
            })
    }

    /// Enabled data section declarations and their owners.
    pub fn data_sections(
        &self,
    ) -> impl Iterator<Item = (&'a PluginId, &'a DataSectionContribution)> + use<'a> {
        self.iter()
            .filter_map(|(id, contribution)| match contribution {
                Contribution::DataSection(spec) => Some((id, spec)),
                _ => None,
            })
    }

    /// Look up a section by its owning plugin and local name.
    pub fn data_section(&self, plugin_id: &str, name: &str) -> Option<&'a DataSectionContribution> {
        self.data_sections()
            .find(|(owner, spec)| owner.as_str() == plugin_id && spec.section_name == name)
            .map(|(_, spec)| spec)
    }
}

pub(crate) fn find_conflicts<'a>(
    declarations: impl Iterator<Item = (&'a PluginId, &'a Contributions)>,
) -> Vec<Conflict> {
    let mut ids: BTreeMap<&str, ContributionOwner> = BTreeMap::new();
    let mut bindings: BTreeMap<(Option<&str>, &str), ContributionOwner> = BTreeMap::new();
    let mut conflicts = Vec::new();
    for (plugin_id, contributions) in declarations {
        for contribution in contributions.iter() {
            let owner = ContributionOwner {
                plugin_id: plugin_id.clone(),
                contribution_id: contribution.id().to_owned(),
            };
            // Section names belong to their plugin namespace, not the global registry.
            if matches!(contribution, Contribution::DataSection(_)) {
                continue;
            }
            if let Some(first) = ids.get(contribution.id()) {
                conflicts.push(Conflict::DuplicateContributionId {
                    id: contribution.id().to_owned(),
                    first: first.clone(),
                    second: owner.clone(),
                });
            } else {
                ids.insert(contribution.id(), owner.clone());
            }
            if let Contribution::Command(command) = contribution
                && let Some(keybinding) = &command.default_keybinding
            {
                let key = (command.context.as_deref(), keybinding.as_str());
                if let Some(first) = bindings.get(&key) {
                    conflicts.push(Conflict::DuplicateDefaultKeybinding {
                        keybinding: keybinding.clone(),
                        context: command.context.clone(),
                        first: first.clone(),
                        second: owner,
                    });
                } else {
                    bindings.insert(key, owner);
                }
            }
        }
    }
    conflicts
}
