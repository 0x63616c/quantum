use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CommandSpec, Contribution, FormatSpec, Manifest, Plugin, PluginId, SettingKind, SettingSpec,
    SettingValue, id::is_namespaced,
};

/// Collects declarations during [`Plugin::register`].
///
/// Validation happens when building the registry, so plugins cannot accidentally
/// ignore a registration error. Nothing is exposed from a failed build.
#[derive(Debug, Default)]
pub struct Registrar {
    contributions: Vec<Contribution>,
}

impl Registrar {
    /// Append a command declaration.
    pub fn command(&mut self, spec: CommandSpec) {
        self.contributions.push(Contribution::Command(spec));
    }

    /// Append a setting declaration.
    pub fn setting(&mut self, spec: SettingSpec) {
        self.contributions.push(Contribution::Setting(spec));
    }

    /// Append a format declaration.
    pub fn format(&mut self, spec: FormatSpec) {
        self.contributions.push(Contribution::Format(spec));
    }
}

/// An enabled declaration and the plugin that owns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredContribution {
    /// Plugin that registered this declaration.
    pub plugin_id: PluginId,
    /// Data for the host to interpret.
    pub contribution: Contribution,
}

/// A registry build failed without exposing a partial registry.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    /// Two plugins declared the same ID, even if disabled.
    #[error("duplicate plugin id {id:?}")]
    DuplicatePlugin {
        /// Repeated plugin namespace.
        id: PluginId,
    },
    /// Enabled declarations share an ID, including across contribution kinds.
    #[error("duplicate contribution id {id:?} from plugins {first_plugin:?} and {second_plugin:?}")]
    DuplicateContribution {
        /// Repeated contribution ID or setting key.
        id: String,
        /// Plugin that registered the first declaration.
        first_plugin: PluginId,
        /// Plugin that registered the conflicting declaration.
        second_plugin: PluginId,
    },
    /// Enabled commands use the same shortcut in the same context.
    #[error(
        "duplicate shortcut {shortcut:?} in context {context:?} (None is global): commands {first_command:?} and {second_command:?}"
    )]
    DuplicateShortcut {
        /// Repeated shortcut string.
        shortcut: String,
        /// Shared context; `None` is global.
        context: Option<String>,
        /// First command using the shortcut.
        first_command: String,
        /// Conflicting command.
        second_command: String,
    },
    /// A contribution contains an invalid ID or setting declaration.
    #[error("invalid contribution {id:?} from plugin {plugin_id:?}: {reason}")]
    InvalidContribution {
        /// Plugin that supplied the declaration.
        plugin_id: PluginId,
        /// Invalid contribution's ID or setting key.
        id: String,
        /// Explanation of the invalid declaration.
        reason: String,
    },
}

/// Validated plugin metadata and enabled contributions in registration order.
#[derive(Debug)]
pub struct Registry {
    plugins: Vec<Manifest>,
    contributions: Vec<RegisteredContribution>,
}

impl Registry {
    /// Build an immutable registry from plugins in the supplied order.
    ///
    /// All plugin IDs must be unique. Only enabled plugins have `register`
    /// called; their contributions share one ID namespace across commands,
    /// settings and formats. IDs need not start with the owning plugin ID.
    /// Disabled contributions neither appear nor reserve IDs or shortcuts.
    ///
    /// The enabled set is authoritative: the host resolves `default_enabled`
    /// and `can_disable` policy before calling this method. Unknown enabled IDs
    /// are ignored. All manifests remain available through [`Self::plugins`].
    pub fn new(
        plugins: &[&dyn Plugin],
        enabled: &BTreeSet<PluginId>,
    ) -> Result<Self, RegistryError> {
        let mut registry = Self {
            plugins: Vec::with_capacity(plugins.len()),
            contributions: Vec::new(),
        };
        let mut plugin_ids = BTreeSet::new();
        for plugin in plugins {
            let manifest = plugin.manifest();
            if !plugin_ids.insert(manifest.id.clone()) {
                return Err(RegistryError::DuplicatePlugin { id: manifest.id });
            }
            registry.plugins.push(manifest);
        }

        let mut contribution_ids = BTreeMap::new();
        let mut shortcuts = BTreeMap::new();
        for (plugin, manifest) in plugins.iter().zip(&registry.plugins) {
            if !enabled.contains(&manifest.id) {
                continue;
            }
            let mut registrar = Registrar::default();
            plugin.register(&mut registrar);
            for contribution in registrar.contributions {
                let id = contribution.id();
                if let Some(reason) = invalid_reason(&contribution) {
                    return Err(RegistryError::InvalidContribution {
                        plugin_id: manifest.id.clone(),
                        id: id.to_owned(),
                        reason: reason.into(),
                    });
                }
                if let Some(first_plugin) =
                    contribution_ids.insert(id.to_owned(), manifest.id.clone())
                {
                    return Err(RegistryError::DuplicateContribution {
                        id: id.to_owned(),
                        first_plugin,
                        second_plugin: manifest.id.clone(),
                    });
                }
                if let Contribution::Command(command) = &contribution
                    && let Some(shortcut) = &command.shortcut
                    && let Some(first_command) = shortcuts.insert(
                        (command.context.clone(), shortcut.clone()),
                        command.id.clone(),
                    )
                {
                    return Err(RegistryError::DuplicateShortcut {
                        shortcut: shortcut.clone(),
                        context: command.context.clone(),
                        first_command,
                        second_command: command.id.clone(),
                    });
                }
                registry.contributions.push(RegisteredContribution {
                    plugin_id: manifest.id.clone(),
                    contribution,
                });
            }
        }
        Ok(registry)
    }

    /// All manifests, including disabled plugins, in plugin list order.
    pub fn plugins(&self) -> impl ExactSizeIterator<Item = &Manifest> {
        self.plugins.iter()
    }

    /// Enabled declarations in plugin order, then each plugin's registration order.
    pub fn contributions(&self) -> impl ExactSizeIterator<Item = &RegisteredContribution> {
        self.contributions.iter()
    }

    /// Enabled commands in registration order.
    pub fn commands(&self) -> impl Iterator<Item = &CommandSpec> {
        self.contributions
            .iter()
            .filter_map(|entry| match &entry.contribution {
                Contribution::Command(spec) => Some(spec),
                _ => None,
            })
    }

    /// Enabled settings in registration order.
    pub fn settings(&self) -> impl Iterator<Item = &SettingSpec> {
        self.contributions
            .iter()
            .filter_map(|entry| match &entry.contribution {
                Contribution::Setting(spec) => Some(spec),
                _ => None,
            })
    }

    /// Enabled formats in registration order.
    pub fn formats(&self) -> impl Iterator<Item = &FormatSpec> {
        self.contributions
            .iter()
            .filter_map(|entry| match &entry.contribution {
                Contribution::Format(spec) => Some(spec),
                _ => None,
            })
    }

    /// Look up an enabled command for host action binding.
    pub fn command(&self, id: &str) -> Option<&CommandSpec> {
        self.commands().find(|spec| spec.id == id)
    }

    /// Look up an enabled setting by key.
    pub fn setting(&self, key: &str) -> Option<&SettingSpec> {
        self.settings().find(|spec| spec.key == key)
    }

    /// Look up an enabled format by ID.
    pub fn format(&self, id: &str) -> Option<&FormatSpec> {
        self.formats().find(|spec| spec.id == id)
    }
}

fn invalid_reason(contribution: &Contribution) -> Option<&'static str> {
    if !is_namespaced(contribution.id()) {
        return Some("expected a lowercase dotted ID with segments matching [a-z][a-z0-9_-]*");
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
            _ => return Some("default value must match the setting kind"),
        }
    }
    None
}
