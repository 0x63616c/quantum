use crate::PluginId;

/// Plugin identity and host-facing enablement policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    /// Stable plugin namespace.
    pub id: PluginId,
    /// Display name.
    pub name: String,
    /// Description shown by the host.
    pub description: String,
    /// Plugin version, independent of the document format version.
    pub version: String,
    /// Whether this plugin ships with Voxelite.
    pub builtin: bool,
    /// Initial preference used by the host when resolving the enabled set.
    pub default_enabled: bool,
    /// Whether the host may let the user disable this plugin.
    pub can_disable: bool,
}

/// Command metadata; execution is bound by the host using `id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    /// Stable namespaced ID, using the same grammar as [`PluginId`].
    pub id: String,
    /// Display label.
    pub label: String,
    /// Optional canonical host shortcut string, such as `secondary-s`.
    pub shortcut: Option<String>,
    /// Opaque shortcut context; `None` denotes the global context.
    ///
    /// Conflicts compare both context and shortcut strings exactly. Hosts are
    /// responsible for canonical spelling and any overlapping context rules.
    pub context: Option<String>,
    /// Optional host icon identifier.
    pub icon: Option<String>,
    /// Optional host menu identifier or path.
    pub menu: Option<String>,
    /// Whether the host may expose this command to agents.
    pub agent_visible: bool,
}

/// A setting's supported value type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingKind {
    /// A boolean toggle.
    Bool,
    /// An ordered, nonempty list of distinct, nonempty choice values.
    Choice(Vec<String>),
}

/// A typed setting value, used for defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingValue {
    /// A boolean value.
    Bool(bool),
    /// One of the declared choice values.
    Choice(String),
}

/// A setting declaration that hosts can render without plugin-specific UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingSpec {
    /// Stable namespaced key, using the same grammar as [`PluginId`].
    pub key: String,
    /// Display label.
    pub label: String,
    /// Supported value type and, for choices, available values.
    pub kind: SettingKind,
    /// Initial value; must match `kind` and any declared choices.
    pub default: SettingValue,
}

/// Metadata for a host-bound file format implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatSpec {
    /// Stable namespaced ID, using the same grammar as [`PluginId`].
    pub id: String,
    /// Display label.
    pub label: String,
    /// File extensions without leading dots, in display order.
    pub extensions: Vec<String>,
    /// Whether the format supports importing.
    pub import: bool,
    /// Whether the format supports exporting.
    pub export: bool,
}

/// One data-only declaration, retained in registration order across all kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Contribution {
    /// A command declaration.
    Command(CommandSpec),
    /// A setting declaration.
    Setting(SettingSpec),
    /// A file format declaration.
    Format(FormatSpec),
}

impl Contribution {
    /// The contribution ID or setting key in the shared contribution namespace.
    pub fn id(&self) -> &str {
        match self {
            Self::Command(spec) => &spec.id,
            Self::Setting(spec) => &spec.key,
            Self::Format(spec) => &spec.id,
        }
    }
}
