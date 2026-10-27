use crate::{PluginId, Value, Version};

/// Plugin identity and host-facing enablement policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manifest {
    /// Stable reverse-DNS plugin namespace.
    pub id: PluginId,
    /// Localization key for the display name.
    pub name_key: String,
    /// Localization key for the description shown by the host.
    pub description_key: String,
    /// Plugin version, independent of the API and document format versions.
    pub version: Version,
    /// API contract used by this plugin, normally [`crate::API_VERSION`].
    pub api_version: Version,
    /// Whether this plugin ships with Voxelite.
    pub builtin: bool,
    /// Initial preference used by the host when resolving the enabled set.
    pub default_enabled: bool,
    /// Whether the host may let the user disable this plugin.
    pub can_disable: bool,
}

/// Command metadata; execution is bound by the host using `id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandContribution {
    /// Stable namespaced ID; see [`Contribution::id`].
    pub id: String,
    /// Localization key for the display label.
    pub label_key: String,
    /// JSON Schema describing typed invocation arguments.
    ///
    /// Retained verbatim; schema interpretation and argument validation belong
    /// to the host. Use an object schema with no properties for no arguments.
    pub args_schema: Value,
    /// Optional canonical host keybinding string, such as `secondary-s`.
    pub default_keybinding: Option<String>,
    /// Opaque keybinding context; `None` denotes the global context.
    pub context: Option<String>,
    /// Optional host icon identifier.
    pub icon: Option<String>,
    /// Optional host menu identifier or path.
    pub menu: Option<String>,
    /// Whether the host may expose this command to agents.
    pub agent_visible: bool,
}

/// Metadata for a host-bound interactive viewport tool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolContribution {
    /// Stable namespaced ID; see [`Contribution::id`].
    pub id: String,
    /// Localization key for the tool label.
    pub label_key: String,
    /// Optional host icon identifier.
    pub icon: Option<String>,
}

/// Metadata for a panel whose presentation is supplied by the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelContribution {
    /// Stable namespaced ID; see [`Contribution::id`].
    pub id: String,
    /// Localization key for the panel title.
    pub label_key: String,
    /// Optional host icon identifier.
    pub icon: Option<String>,
}

/// Metadata for a host-bound file format implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatContribution {
    /// Stable namespaced ID; see [`Contribution::id`].
    pub id: String,
    /// Localization key for the display label.
    pub label_key: String,
    /// File extensions without leading dots, in display order.
    pub extensions: Vec<String>,
    /// Whether the format supports importing.
    pub import: bool,
    /// Whether the format supports exporting.
    pub export: bool,
}

/// A declarative insertion point relative to the core scene renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderPassSlot {
    /// Preparation before core scene rendering, such as shadow data.
    BeforeScene,
    /// Rendering after the core scene, before overlays.
    AfterScene,
    /// Viewport overlays, such as a grid or gizmo.
    Overlay,
}

/// Metadata for a host-bound render pass; contains no renderer resources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPassContribution {
    /// Stable namespaced ID; see [`Contribution::id`].
    pub id: String,
    /// Requested insertion point; scheduling is the host's responsibility.
    pub slot: RenderPassSlot,
}

/// A setting's supported value type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingKind {
    /// A boolean toggle.
    Bool,
    /// An ordered, nonempty list of distinct, nonempty choice values.
    Choice(Vec<String>),
    /// A finite floating-point number.
    Number,
}

/// A typed setting value, used for defaults.
#[derive(Clone, Debug, PartialEq)]
pub enum SettingValue {
    /// A boolean value.
    Bool(bool),
    /// One of the declared choice values.
    Choice(String),
    /// A finite numeric value.
    Number(f64),
}

/// A setting schema that hosts can render without plugin-specific UI.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingContribution {
    /// Stable namespaced key; see [`Contribution::id`].
    pub key: String,
    /// Localization key for the display label.
    pub label_key: String,
    /// Supported value type and, for choices, available values.
    pub kind: SettingKind,
    /// Initial value; must match `kind` and any declared choices.
    pub default: SettingValue,
}

/// Declaration of a plugin-owned `.voxl` data section.
///
/// This metadata does not change the file format or implement serialization,
/// migration or ownership checks against core sections; those belong to the host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataSectionContribution {
    /// Stable section name, such as `comments` or `dev.voxelite.comments.pins`.
    ///
    /// Each dot-separated segment matches `[a-z][a-z0-9_-]*`.
    pub section_name: String,
    /// Section schema revision, independent of plugin, API and `.voxl` versions.
    pub schema_version: u32,
}

/// One data-only declaration, retained in registration order across all kinds.
#[derive(Clone, Debug, PartialEq)]
pub enum Contribution {
    /// A command declaration.
    Command(CommandContribution),
    /// An interactive tool declaration.
    Tool(ToolContribution),
    /// A panel declaration.
    Panel(PanelContribution),
    /// A file format declaration.
    Format(FormatContribution),
    /// A render pass declaration.
    RenderPass(RenderPassContribution),
    /// A setting declaration.
    Setting(SettingContribution),
    /// A `.voxl` data section declaration.
    DataSection(DataSectionContribution),
}

impl Contribution {
    /// The contribution ID, setting key or data section name.
    ///
    /// IDs and keys use at least two dot-separated segments matching
    /// `[a-z][a-z0-9_-]*`, for example `dev.voxelite.tools.paint`.
    /// Data sections also allow a single segment to describe existing `.voxl`
    /// sections. All kinds share one identifier namespace within a plugin.
    pub fn id(&self) -> &str {
        match self {
            Self::Command(spec) => &spec.id,
            Self::Tool(spec) => &spec.id,
            Self::Panel(spec) => &spec.id,
            Self::Format(spec) => &spec.id,
            Self::RenderPass(spec) => &spec.id,
            Self::Setting(spec) => &spec.key,
            Self::DataSection(spec) => &spec.section_name,
        }
    }
}
