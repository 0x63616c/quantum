#[path = "export.rs"]
mod export;
pub use export::{ExportCompletion, ExportJob, SnapshotExport};

use std::{
    any::Any,
    collections::{BTreeMap, BTreeSet},
    panic::{AssertUnwindSafe, catch_unwind},
};

use voxelite_plugin_api::{
    API_VERSION, ActivateCx, Command, Contribution, Contributions, Manifest, OverlayCx,
    OverlayFrame, OverlayInput, OverlayOutcome, OverlayPrimitive, Plugin, PluginId, Region,
    Registrar, SettingKind, SettingValue, ToolCx, ToolError, ToolInput, ToolOutcome, TransformCx,
    ValidationError, Version,
};

use crate::{BudgetExceeded, Conflict, Registry, Timings, registry::find_conflicts};

/// Serializable preference boundary, independent of any persistence format.
///
/// Conversions preserve unknown IDs so callers can round-trip preferences before
/// all plugins are available. The host ignores unknown IDs when loading a set.
/// [`Host::enabled_set`] returns only plugins that are actually enabled.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnabledSet(BTreeSet<String>);

impl EnabledSet {
    /// Whether an ID is in the saved set.
    pub fn contains(&self, id: &str) -> bool {
        self.0.contains(id)
    }

    /// Borrow the IDs in sorted order for persistence.
    pub fn as_set(&self) -> &BTreeSet<String> {
        &self.0
    }

    /// Serialize to a sorted, deduplicated vector of IDs.
    pub fn to_vec(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }
}

impl From<Vec<String>> for EnabledSet {
    fn from(ids: Vec<String>) -> Self {
        Self(ids.into_iter().collect())
    }
}

impl From<BTreeSet<String>> for EnabledSet {
    fn from(ids: BTreeSet<String>) -> Self {
        Self(ids)
    }
}

impl From<EnabledSet> for Vec<String> {
    fn from(ids: EnabledSet) -> Self {
        ids.0.into_iter().collect()
    }
}

impl From<EnabledSet> for BTreeSet<String> {
    fn from(ids: EnabledSet) -> Self {
        ids.0
    }
}

/// A plugin entry point guarded by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Collecting declarations.
    Contribute,
    /// Initializing an enabled plugin.
    Activate,
    /// Releasing an enabled plugin's state.
    Deactivate,
    /// Binding or invoking a tool through read-only capabilities.
    Tool,
    /// Binding or invoking a pending command transform.
    CommandTransform,
    /// Binding, drawing or interacting with a portable overlay.
    Overlay,
    /// Binding or invoking a read-only exporter.
    Export,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Self::Contribute => "contribute",
            Self::Activate => "activate",
            Self::Deactivate => "deactivate",
            Self::Tool => "tool",
            Self::CommandTransform => "command_transform",
            Self::Overlay => "overlay",
            Self::Export => "export",
        }
    }
}

/// Lifecycle state of an admitted plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginState {
    /// Valid declarations are retained privately, but none are published.
    Disabled,
    /// Activation succeeded and declarations are published.
    Enabled,
    /// A hook panicked; declarations were discarded and no hooks will be retried.
    Faulted {
        /// Hook that panicked.
        phase: Phase,
        /// String panic payload, or a description of a non-string payload.
        message: String,
    },
}

/// A manifest fails the host's metadata or API compatibility checks.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ManifestError {
    /// A required localization key is blank.
    #[error("manifest {field} must not be blank")]
    EmptyLocalizationKey {
        /// Name of the manifest field.
        field: &'static str,
    },
    /// The plugin requires an API contract this host cannot provide.
    #[error("required API {required} is incompatible with host API {available}")]
    IncompatibleApi {
        /// Plugin's required API version.
        required: Version,
        /// Host's API version.
        available: Version,
    },
}

/// A rejected registration; other plugins can still be registered.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RegistrationError {
    /// `manifest()` panicked before the host could establish an identity.
    #[error("plugin manifest panicked: {message}")]
    ManifestPanicked {
        /// Panic payload description.
        message: String,
    },
    /// Another plugin already owns this ID, even if it is disabled or faulted.
    #[error("duplicate plugin id {plugin_id}")]
    DuplicatePluginId {
        /// Rejected ID.
        plugin_id: PluginId,
    },
    /// Manifest metadata or API version is invalid.
    #[error("invalid manifest for {plugin_id}: {error}")]
    InvalidManifest {
        /// Rejected plugin.
        plugin_id: PluginId,
        /// Validation failure.
        error: ManifestError,
    },
    /// No declarations from this plugin were admitted.
    #[error("invalid contributions for {plugin_id}: {error}")]
    InvalidContributions {
        /// Rejected plugin.
        plugin_id: PluginId,
        /// Local declaration validation failure.
        error: ValidationError,
    },
}

/// A runtime lifecycle, setting, tool or command transform request failed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum HostError {
    /// Export encoding failed without changing the document.
    #[error("export failed: {message}")]
    Export {
        /// Encoder error or unavailable format detail.
        message: String,
    },
    /// A runtime setting value does not match its declared schema.
    #[error("invalid value for plugin setting {key:?}")]
    InvalidSetting {
        /// Declared setting key.
        key: String,
    },
    /// A tool rejected a gesture without changing the document.
    #[error(transparent)]
    Tool(#[from] ToolError),
    /// No admitted plugin has this identity.
    #[error("unknown plugin {id:?}")]
    UnknownPlugin {
        /// Requested ID.
        id: String,
    },
    /// The manifest forbids user disablement.
    #[error("plugin {plugin_id} cannot be disabled")]
    CannotDisable {
        /// Required plugin.
        plugin_id: PluginId,
    },
    /// The plugin's declarations conflict with each other or enabled plugins.
    #[error("plugin {plugin_id} has contribution conflicts: {conflicts:?}")]
    Conflicts {
        /// Plugin that remains disabled.
        plugin_id: PluginId,
        /// Every conflict in registration order.
        conflicts: Vec<Conflict>,
    },
    /// A panicking plugin is quarantined for the lifetime of this host.
    #[error("plugin {plugin_id} faulted during {phase:?}: {message}")]
    Faulted {
        /// Faulted plugin.
        plugin_id: PluginId,
        /// Hook that panicked.
        phase: Phase,
        /// Panic payload description.
        message: String,
    },
}

/// Notices the shell can consume without inspecting plugin implementations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostEvent {
    /// Registration failed before publishing any declarations.
    RegistrationRejected(RegistrationError),
    /// A plugin successfully became enabled.
    Enabled(PluginId),
    /// A plugin successfully became disabled.
    Disabled(PluginId),
    /// Enablement was blocked by declaration conflicts.
    EnableRejected {
        /// Plugin that remains disabled.
        plugin_id: PluginId,
        /// Conflicting declarations.
        conflicts: Vec<Conflict>,
    },
    /// A plugin was quarantined and its declarations discarded.
    Faulted {
        /// Faulted plugin.
        plugin_id: PluginId,
        /// Hook that panicked.
        phase: Phase,
        /// Panic payload description.
        message: String,
    },
    /// An operation exceeded the configured advisory timing budget.
    BudgetExceeded(BudgetExceeded),
}

/// Test whether a required API version is compatible with an available version.
///
/// Requirements may not be newer than the host. Before 1.0, major and minor must
/// match; from 1.0, major must match. Prereleases require identical version
/// precedence, since their compatibility is not promised. Build metadata never
/// affects compatibility.
pub fn api_compatible(required: &Version, available: &Version) -> bool {
    if !required.pre.is_empty() || !available.pre.is_empty() {
        return required.cmp_precedence(available).is_eq();
    }
    required.major == available.major
        && (required.major != 0 || required.minor == available.minor)
        && !required.cmp_precedence(available).is_gt()
}

/// Validate required localization keys and compatibility with [`API_VERSION`].
///
/// Plugin ID syntax is already guaranteed by [`PluginId`]. Duplicate plugin IDs
/// are checked by [`Host::register`]. Implementation versions are independent.
pub fn validate_manifest(manifest: &Manifest) -> Result<(), ManifestError> {
    for (field, key) in [
        ("name_key", &manifest.name_key),
        ("description_key", &manifest.description_key),
    ] {
        if key.trim().is_empty() {
            return Err(ManifestError::EmptyLocalizationKey { field });
        }
    }
    if !api_compatible(&manifest.api_version, &API_VERSION) {
        return Err(ManifestError::IncompatibleApi {
            required: manifest.api_version.clone(),
            available: API_VERSION,
        });
    }
    Ok(())
}

pub(crate) struct HostedPlugin {
    plugin: Box<dyn Plugin>,
    pub(crate) manifest: Manifest,
    pub(crate) state: PluginState,
    pub(crate) contributions: Contributions,
    settings: BTreeMap<String, SettingValue>,
}

/// Owns plugin instances and the read-only registry they populate.
///
/// Registration and activation proceed in supplied order. Failed registrations
/// and initial activations produce drainable events without stopping the list.
/// Declarations are collected once, retained while disabled, and published only
/// after successful activation. Conflicts block the whole candidate plugin.
#[derive(Default)]
pub struct Host {
    export_authority: std::sync::Arc<()>,
    plugins: Vec<HostedPlugin>,
    events: Vec<HostEvent>,
    timings: Timings,
}

impl Host {
    /// Draw enabled render-pass overlay bindings in registration order.
    /// A fault discards that plugin's entire frame; other plugins keep drawing.
    pub fn draw_overlays(
        &mut self,
        bounds: Region,
        camera_basis: [[f64; 3]; 3],
        hovered: Option<(&str, u32)>,
    ) -> Vec<OverlayFrame> {
        let passes: Vec<_> = self
            .registry()
            .render_passes()
            .map(|(_, pass)| pass.clone())
            .collect();
        let mut frames = Vec::new();
        for pass in passes {
            let Some(index) = self.overlay_owner(&pass.id) else {
                continue;
            };
            let mut cx = OverlayContext {
                bounds,
                camera_basis,
                settings: self.plugins[index].settings.clone(),
                primitives: Vec::new(),
            };
            let target = hovered
                .filter(|(id, _)| *id == pass.id)
                .map(|(_, target)| target);
            let drawn = self.invoke(index, Phase::Overlay, |plugin| {
                plugin
                    .overlay(&pass.id)
                    .map(|overlay| overlay.draw(&mut cx, target))
                    .is_some()
            });
            if matches!(drawn, Ok(true)) {
                frames.push(OverlayFrame {
                    id: pass.id,
                    slot: pass.slot,
                    primitives: cx.primitives,
                });
            }
        }
        // A later pass can fault the same owner after an earlier pass succeeded.
        frames.retain(|frame| self.overlay_owner(&frame.id).is_some());
        frames
    }

    /// Dispatch a declared enabled overlay's pointer input without document mutation.
    pub fn dispatch_overlay(
        &mut self,
        id: &str,
        bounds: Region,
        camera_basis: [[f64; 3]; 3],
        input: OverlayInput,
    ) -> Result<Option<OverlayOutcome>, HostError> {
        let Some(index) = self.overlay_owner(id) else {
            return Ok(None);
        };
        let cx = OverlayContext {
            bounds,
            camera_basis,
            settings: self.plugins[index].settings.clone(),
            primitives: Vec::new(),
        };
        let mut outcome = self
            .invoke(index, Phase::Overlay, |plugin| {
                plugin
                    .overlay(id)
                    .map(|overlay| overlay.interact(&cx, input))
            })?
            .transpose()?;
        if let Some(outcome) = &mut outcome
            && !outcome.commands.is_empty()
        {
            outcome.commands =
                self.transform_commands(bounds, std::mem::take(&mut outcome.commands))?;
        }
        Ok(outcome)
    }

    /// Whether an enabled render-pass declaration has a portable overlay binding.
    pub fn has_overlay(&mut self, id: &str) -> Result<bool, HostError> {
        let Some(index) = self.overlay_owner(id) else {
            return Ok(false);
        };
        self.invoke(index, Phase::Overlay, |plugin| plugin.overlay(id).is_some())
    }

    fn overlay_owner(&self, id: &str) -> Option<usize> {
        self.plugins.iter().position(|plugin| {
            plugin.state == PluginState::Enabled
                && plugin
                    .contributions
                    .iter()
                    .any(|c| matches!(c, Contribution::RenderPass(pass) if pass.id == id))
        })
    }

    /// Resolve and invoke a declared, enabled tool. Missing or disabled tools are inert.
    /// Transforms run on each returned outcome before any commands reach the app.
    /// Binding and execution are timed and panic-isolated like lifecycle hooks.
    pub fn dispatch_tool(
        &mut self,
        id: &str,
        cx: &dyn ToolCx,
        input: ToolInput,
    ) -> Result<Option<ToolOutcome>, HostError> {
        let Some(index) = self.tool_owner(id) else {
            return Ok(None);
        };
        let mut outcome = self
            .invoke(index, Phase::Tool, |plugin| {
                plugin.tool(id).map(|tool| tool.interact(cx, input))
            })?
            .transpose()
            .map_err(HostError::from)?;
        if let Some(outcome) = &mut outcome {
            outcome.commands =
                self.transform_commands(cx.bounds(), std::mem::take(&mut outcome.commands))?;
        }
        Ok(outcome)
    }

    /// Whether an enabled tool declaration has an implementation binding.
    pub fn has_tool(&mut self, id: &str) -> Result<bool, HostError> {
        let Some(index) = self.tool_owner(id) else {
            return Ok(false);
        };
        self.invoke(index, Phase::Tool, |plugin| plugin.tool(id).is_some())
    }

    fn tool_owner(&self, id: &str) -> Option<usize> {
        self.plugins.iter().position(|plugin| {
            plugin.state == PluginState::Enabled
                && plugin
                    .contributions
                    .iter()
                    .any(|c| matches!(c, Contribution::Tool(tool) if tool.id == id))
        })
    }

    /// Run enabled transforms in ascending order, with registration order for ties.
    /// Missing bindings are inert; a panic aborts the pending edit and quarantines
    /// its plugin. No document mutation occurs here.
    pub fn transform_commands(
        &mut self,
        bounds: Region,
        mut commands: Vec<Command>,
    ) -> Result<Vec<Command>, HostError> {
        let mut transforms: Vec<_> = self
            .plugins
            .iter()
            .enumerate()
            .filter(|(_, plugin)| plugin.state == PluginState::Enabled)
            .flat_map(|(index, plugin)| {
                plugin.contributions.iter().filter_map(move |c| {
                    if let Contribution::CommandTransform(spec) = c {
                        Some((index, spec.id.clone(), spec.order))
                    } else {
                        None
                    }
                })
            })
            .collect();
        transforms.sort_by_key(|(_, _, order)| *order);
        for (index, id, _) in transforms {
            let cx = TransformContext {
                bounds,
                settings: self.plugins[index].settings.clone(),
            };
            commands = self.invoke(index, Phase::CommandTransform, |plugin| {
                match plugin.command_transform(&id) {
                    Some(transform) => transform.transform(&cx, commands),
                    None => commands,
                }
            })?;
        }
        Ok(commands)
    }

    /// Whether a declared, enabled command transform has an implementation binding.
    pub fn has_command_transform(&mut self, id: &str) -> Result<bool, HostError> {
        let Some(index) = self.plugins.iter().position(|plugin| {
            plugin.state == PluginState::Enabled
                && plugin
                    .contributions
                    .iter()
                    .any(|c| matches!(c, Contribution::CommandTransform(spec) if spec.id == id))
        }) else {
            return Ok(false);
        };
        self.invoke(index, Phase::CommandTransform, |plugin| {
            plugin.command_transform(id).is_some()
        })
    }

    /// Read an enabled plugin's setting. Values survive disable/enable in this host;
    /// serialization, if any, remains the app's responsibility.
    pub fn setting(&self, key: &str) -> Option<&SettingValue> {
        self.plugins
            .iter()
            .filter(|plugin| plugin.state == PluginState::Enabled)
            .find_map(|plugin| plugin.settings.get(key))
    }

    /// Update declared plugin state without affecting document content or history.
    /// Unknown, disabled and faulted settings are inert and return `false`.
    pub fn set_setting(&mut self, key: &str, value: SettingValue) -> Result<bool, HostError> {
        for plugin in &mut self.plugins {
            if plugin.state != PluginState::Enabled {
                continue;
            }
            let Some(setting) = plugin.contributions.iter().find_map(|c| match c {
                Contribution::Setting(setting) if setting.key == key => Some(setting),
                _ => None,
            }) else {
                continue;
            };
            let valid = match (&setting.kind, &value) {
                (SettingKind::Bool, SettingValue::Bool(_)) => true,
                (SettingKind::Choice(choices), SettingValue::Choice(value)) => {
                    choices.contains(value)
                }
                (SettingKind::Number, SettingValue::Number(value)) => value.is_finite(),
                _ => false,
            };
            if !valid {
                return Err(HostError::InvalidSetting { key: key.into() });
            }
            let changed = plugin.settings.get(key) != Some(&value);
            plugin.settings.insert(key.into(), value);
            return Ok(changed);
        }
        Ok(false)
    }

    /// Register plugins and activate those enabled by their manifests.
    pub fn new(plugins: Vec<Box<dyn Plugin>>) -> Self {
        let mut host = Self::default();
        host.register(plugins);
        host
    }

    /// Register plugins using an explicit saved set instead of manifest defaults.
    ///
    /// An empty set disables all optional plugins. Required plugins are always
    /// selected, subject to validation, conflicts and successful activation.
    pub fn with_enabled(plugins: Vec<Box<dyn Plugin>>, enabled: EnabledSet) -> Self {
        let mut host = Self::default();
        host.register_selected(plugins, Some(&enabled));
        host
    }

    /// Append plugins in order, applying their manifest enablement defaults.
    ///
    /// Errors are reported through [`Self::drain_events`]. Duplicate plugin IDs
    /// never replace the first instance. Disabled plugins still declare and
    /// validate contributions so catalog conflicts are visible before enablement.
    pub fn register(&mut self, plugins: Vec<Box<dyn Plugin>>) {
        self.register_selected(plugins, None);
    }

    fn register_selected(&mut self, plugins: Vec<Box<dyn Plugin>>, enabled: Option<&EnabledSet>) {
        for plugin in plugins {
            let manifest = match isolate(|| plugin.manifest()) {
                Ok(manifest) => manifest,
                Err(message) => {
                    self.reject(RegistrationError::ManifestPanicked { message });
                    continue;
                }
            };
            let id = manifest.id.clone();
            if self.state(id.as_str()).is_some() {
                self.reject(RegistrationError::DuplicatePluginId { plugin_id: id });
                continue;
            }
            if let Err(error) = validate_manifest(&manifest) {
                self.reject(RegistrationError::InvalidManifest {
                    plugin_id: id,
                    error,
                });
                continue;
            }
            let selected = !manifest.can_disable
                || enabled.map_or(manifest.default_enabled, |set| set.contains(id.as_str()));
            let index = self.plugins.len();
            self.plugins.push(HostedPlugin {
                plugin,
                manifest,
                state: PluginState::Disabled,
                contributions: Contributions::default(),
                settings: BTreeMap::new(),
            });
            match self.collect(index) {
                Ok(Ok(contributions)) => {
                    self.plugins[index].settings = contributions
                        .iter()
                        .filter_map(|c| match c {
                            Contribution::Setting(setting) => {
                                Some((setting.key.clone(), setting.default.clone()))
                            }
                            _ => None,
                        })
                        .collect();
                    self.plugins[index].contributions = contributions;
                }
                Ok(Err(error)) => {
                    self.plugins.pop();
                    self.reject(RegistrationError::InvalidContributions {
                        plugin_id: id,
                        error,
                    });
                    continue;
                }
                Err(_) => continue, // The invocation already faulted and emitted a notice.
            }
            if selected {
                let _ = self.enable(id.as_str()); // Failure is also a shell notice.
            }
        }
    }

    /// Read enabled declarations without exposing plugin instances.
    pub fn registry(&self) -> Registry<'_> {
        Registry {
            plugins: &self.plugins,
        }
    }

    /// All admitted manifests and states, including disabled and faulted plugins.
    pub fn plugins(&self) -> impl Iterator<Item = (&Manifest, &PluginState)> {
        self.plugins
            .iter()
            .map(|plugin| (&plugin.manifest, &plugin.state))
    }

    /// Look up an admitted plugin's current state.
    pub fn state(&self, id: &str) -> Option<&PluginState> {
        self.plugins
            .iter()
            .find(|plugin| plugin.manifest.id.as_str() == id)
            .map(|plugin| &plugin.state)
    }

    /// Catalog conflicts, including disabled plugins, in registration order.
    ///
    /// With three declarations sharing a key, each later declaration is reported
    /// against the first. Faulted plugins have no declarations to conflict with.
    /// Keybindings are canonical API strings, compared verbatim within a context;
    /// global and named contexts are distinct. Conflicts within a plugin's own
    /// keybindings are reported too.
    pub fn conflicts(&self) -> Vec<Conflict> {
        find_conflicts(
            self.plugins
                .iter()
                .map(|plugin| (&plugin.manifest.id, &plugin.contributions)),
        )
    }

    /// Enable a plugin, publishing its declarations after activation succeeds.
    ///
    /// Already-enabled requests are no-ops. Conflicting declarations prevent
    /// activation and leave the plugin disabled; callers can retry after disabling
    /// conflicting plugins. Faulted plugins cannot be retried.
    pub fn enable(&mut self, id: &str) -> Result<(), HostError> {
        let index = self.index(id)?;
        self.ensure_healthy(index)?;
        if self.plugins[index].state == PluginState::Enabled {
            return Ok(());
        }
        let conflicts = find_conflicts(self.plugins.iter().enumerate().filter_map(
            |(other, plugin)| {
                (other == index || plugin.state == PluginState::Enabled)
                    .then_some((&plugin.manifest.id, &plugin.contributions))
            },
        ));
        let plugin_id = self.plugins[index].manifest.id.clone();
        if !conflicts.is_empty() {
            self.events.push(HostEvent::EnableRejected {
                plugin_id: plugin_id.clone(),
                conflicts: conflicts.clone(),
            });
            return Err(HostError::Conflicts {
                plugin_id,
                conflicts,
            });
        }
        self.invoke(index, Phase::Activate, |plugin| {
            plugin.activate(&mut ActivationContext);
        })?;
        self.plugins[index].state = PluginState::Enabled;
        self.events.push(HostEvent::Enabled(plugin_id));
        Ok(())
    }

    /// Disable an optional plugin, immediately removing its declarations.
    ///
    /// Already-disabled requests are no-ops. Required plugins reject the request
    /// without invoking code. A panicking deactivation still removes declarations.
    pub fn disable(&mut self, id: &str) -> Result<(), HostError> {
        let index = self.index(id)?;
        self.ensure_healthy(index)?;
        if !self.plugins[index].manifest.can_disable {
            return Err(HostError::CannotDisable {
                plugin_id: self.plugins[index].manifest.id.clone(),
            });
        }
        self.deactivate(index)
    }

    /// Snapshot actual enabled IDs for later application-owned persistence.
    pub fn enabled_set(&self) -> EnabledSet {
        EnabledSet(
            self.plugins
                .iter()
                .filter(|plugin| plugin.state == PluginState::Enabled)
                .map(|plugin| plugin.manifest.id.to_string())
                .collect(),
        )
    }

    /// Measurements of contribution and lifecycle hooks, including failed calls.
    /// Manifest calls have no established plugin identity and are not measured.
    pub fn timings(&self) -> &Timings {
        &self.timings
    }

    /// Configure the budget or time other host-owned plugin operations.
    pub fn timings_mut(&mut self) -> &mut Timings {
        &mut self.timings
    }

    /// Take pending lifecycle, rejection, fault and budget notices.
    pub fn drain_events(&mut self) -> Vec<HostEvent> {
        self.collect_budget_notices();
        std::mem::take(&mut self.events)
    }

    // The conformance harness also exercises required plugins' cleanup hooks.
    // Public disablement always checks can_disable before reaching this method.
    pub(crate) fn deactivate(&mut self, index: usize) -> Result<(), HostError> {
        self.ensure_healthy(index)?;
        if self.plugins[index].state == PluginState::Disabled {
            return Ok(());
        }
        self.plugins[index].state = PluginState::Disabled;
        self.invoke(index, Phase::Deactivate, |plugin| plugin.deactivate())?;
        self.events
            .push(HostEvent::Disabled(self.plugins[index].manifest.id.clone()));
        Ok(())
    }

    pub(crate) fn collect(
        &mut self,
        index: usize,
    ) -> Result<Result<Contributions, ValidationError>, HostError> {
        self.invoke(index, Phase::Contribute, |plugin| {
            let mut registrar = Registrar::default();
            plugin.contribute(&mut registrar);
            registrar.finish()
        })
    }

    fn index(&self, id: &str) -> Result<usize, HostError> {
        self.plugins
            .iter()
            .position(|plugin| plugin.manifest.id.as_str() == id)
            .ok_or_else(|| HostError::UnknownPlugin { id: id.to_owned() })
    }

    fn ensure_healthy(&self, index: usize) -> Result<(), HostError> {
        let plugin = &self.plugins[index];
        if let PluginState::Faulted { phase, message } = &plugin.state {
            return Err(HostError::Faulted {
                plugin_id: plugin.manifest.id.clone(),
                phase: *phase,
                message: message.clone(),
            });
        }
        Ok(())
    }

    fn invoke<T>(
        &mut self,
        index: usize,
        phase: Phase,
        operation: impl FnOnce(&mut dyn Plugin) -> T,
    ) -> Result<T, HostError> {
        self.ensure_healthy(index)?;
        let plugin = &mut self.plugins[index];
        let plugin_id = plugin.manifest.id.clone();
        let result = self.timings.time(plugin_id.as_str(), phase.label(), || {
            // The mutable plugin may be inconsistent after unwinding. On error it
            // is quarantined below and never called again, so no UnwindSafe bound
            // is required of plugin authors.
            isolate(|| operation(plugin.plugin.as_mut()))
        });
        self.collect_budget_notices();
        result.map_err(|message| {
            let plugin = &mut self.plugins[index];
            plugin.state = PluginState::Faulted {
                phase,
                message: message.clone(),
            };
            plugin.contributions = Contributions::default();
            self.events.push(HostEvent::Faulted {
                plugin_id: plugin_id.clone(),
                phase,
                message: message.clone(),
            });
            HostError::Faulted {
                plugin_id,
                phase,
                message,
            }
        })
    }

    fn reject(&mut self, error: RegistrationError) {
        self.events.push(HostEvent::RegistrationRejected(error));
    }

    fn collect_budget_notices(&mut self) {
        self.events.extend(
            self.timings
                .drain_notices()
                .into_iter()
                .map(HostEvent::BudgetExceeded),
        );
    }
}

struct ActivationContext;
impl ActivateCx for ActivationContext {}

struct TransformContext {
    bounds: Region,
    settings: BTreeMap<String, SettingValue>,
}

impl TransformCx for TransformContext {
    fn bounds(&self) -> Region {
        self.bounds
    }
    fn setting(&self, key: &str) -> Option<&SettingValue> {
        self.settings.get(key)
    }
}

fn isolate<T>(operation: impl FnOnce() -> T) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(operation)).map_err(panic_message)
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else {
        "non-string panic payload".to_owned()
    }
}

struct OverlayContext {
    bounds: Region,
    camera_basis: [[f64; 3]; 3],
    settings: BTreeMap<String, SettingValue>,
    primitives: Vec<OverlayPrimitive>,
}
impl OverlayCx for OverlayContext {
    fn bounds(&self) -> Region {
        self.bounds
    }
    fn camera_basis(&self) -> [[f64; 3]; 3] {
        self.camera_basis
    }
    fn setting(&self, key: &str) -> Option<&SettingValue> {
        self.settings.get(key)
    }
    fn emit(&mut self, primitive: OverlayPrimitive) {
        self.primitives.push(primitive);
    }
}
