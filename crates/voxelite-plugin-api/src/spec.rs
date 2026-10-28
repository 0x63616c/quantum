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

impl Manifest {
    /// Construct an enabled, disableable built-in using the current API contract.
    ///
    /// Supply a constant ID, localization key prefix and package version.
    /// Required infrastructure may override `can_disable` on the returned value.
    ///
    /// # Panics
    /// Panics with the plugin ID if the ID or semantic version is invalid.
    #[expect(
        clippy::panic,
        reason = "built-in manifest constants must be valid; invalid declarations are programming errors"
    )]
    pub fn builtin(id: &str, key_prefix: &str, version: &str) -> Self {
        Self {
            id: PluginId::new(id)
                .unwrap_or_else(|error| panic!("invalid built-in plugin ID {id:?}: {error}")),
            name_key: format!("{key_prefix}.name"),
            description_key: format!("{key_prefix}.description"),
            version: Version::parse(version).unwrap_or_else(|error| {
                panic!("invalid version for built-in plugin {id:?}: {error}")
            }),
            api_version: crate::API_VERSION,
            builtin: true,
            default_enabled: true,
            can_disable: true,
        }
    }
}

/// Placement within the native application's menu, independent of its usual menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationMenuSection {
    /// Application information and updates.
    About,
    /// Application preferences.
    Settings,
    /// Hide and show application windows.
    Visibility,
    /// Exit the application.
    Quit,
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
    /// Shell presentation order across plugins; ties retain registration order.
    /// This does not change the registry's declaration order.
    pub order: i32,
    /// Optional relocation into the native application's menu on Apple platforms.
    pub application_section: Option<ApplicationMenuSection>,
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

/// Metadata for a rewrite of pending tool commands before transaction admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandTransformContribution {
    /// Stable namespaced ID; see [`Contribution::id`].
    pub id: String,
    /// Execution order across plugins; ties retain registration order.
    pub order: i32,
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

/// Content offered by a format, so shells can build context-specific lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatContent {
    /// A whole document or rendered view.
    Document,
    /// Palette colors in display order.
    Palette,
}

/// Metadata for a host-bound file format implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatContribution {
    /// Content category used to select the appropriate save/export surface.
    pub content: FormatContent,
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

pub use voxelite_render_plan::RenderPassSlot;

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

/// A typed setting value, used for defaults and plugin-scoped runtime state.
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
/// Stored under `extensions[plugin_id][section_name]`. The host gates access and
/// runs [`crate::Plugin::data_migrations`] before [`crate::Plugin::load_data`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataSectionContribution {
    /// Stable section name, such as `comments` or `dev.voxelite.comments.pins`.
    ///
    /// At most 253 bytes; each dot-separated segment matches `[a-z][a-z0-9_-]*`.
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
    /// A pending command transform declaration.
    CommandTransform(CommandTransformContribution),
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
    /// Data sections also allow a single plugin-local segment. All kinds share
    /// one identifier namespace within a plugin.
    pub fn id(&self) -> &str {
        match self {
            Self::Command(spec) => &spec.id,
            Self::Tool(spec) => &spec.id,
            Self::CommandTransform(spec) => &spec.id,
            Self::Panel(spec) => &spec.id,
            Self::Format(spec) => &spec.id,
            Self::RenderPass(spec) => &spec.id,
            Self::Setting(spec) => &spec.key,
            Self::DataSection(spec) => &spec.section_name,
        }
    }
}
