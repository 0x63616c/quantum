//! Portable palette panel state and requests. The host owns persistence and edits.
use crate::{Command, CoreError, Palette, PaletteEdit, SettingValue, Swatch, Uuid};
use serde::{Deserialize, Serialize};

/// A user-owned palette, independent of a document's stable slots.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryPalette {
    /// Stable local identity.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Ordered opaque swatches.
    pub palette: Palette,
}

/// Custom palettes in their persisted display order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaletteLibrary {
    /// Custom entries; presets are separate and immutable.
    pub entries: Vec<LibraryPalette>,
}

/// Rollback point for an uncommitted picker gesture. Never serialized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PalettePreview {
    /// Library before the first preview, including before an automatic copy.
    pub library: PaletteLibrary,
    /// Custom selection before the gesture.
    pub custom: Option<Uuid>,
    /// Manager display position before the gesture.
    pub slot: usize,
}

/// Panel selection and working library. The host admits returned state atomically.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaletteState {
    /// Custom library, including transient edits while a preview is active.
    pub library: PaletteLibrary,
    /// Selected custom palette, if any.
    pub custom: Option<Uuid>,
    /// Selected display position in the manager.
    pub slot: usize,
    /// Original library and selection while a picker gesture is active.
    pub preview: Option<PalettePreview>,
    /// Up to sixteen distinct committed sRGB colors, most recent first.
    /// Persisted beside entries in the existing local library bytes.
    pub recent_colors: Vec<[u8; 3]>,
}

/// Origin and editing policy of a palette in the manager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteKind {
    /// Current document colors; edits in the manager create a custom copy.
    Project,
    /// Immutable bundled colors; edits in the manager create a custom copy.
    BuiltIn,
    /// Editable local library entry.
    Custom,
}

/// Stable identity independent of a custom entry's position in the library.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteId {
    /// Project (zero) or built-in index in the source choices.
    Source(usize),
    /// Local library identity.
    Custom(Uuid),
}

/// One source or custom choice supplied by the panel implementation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteChoice {
    /// Stable selection address.
    pub id: PaletteId,
    /// Palette origin.
    pub kind: PaletteKind,
    /// Whether editing requires an automatic custom copy.
    pub read_only: bool,
    /// Display label.
    pub name: String,
    /// Exact swatches in source order.
    pub palette: Palette,
}

/// Read-only presentation data, with no GUI objects or callbacks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteView {
    /// Selected palette's stable identity.
    pub selected: PaletteId,
    /// Selected palette's origin.
    pub kind: PaletteKind,
    /// Whether editing the selection requires an automatic custom copy.
    pub read_only: bool,
    /// Selected palette's display name.
    pub name: String,
    /// Selected palette's original colors.
    pub palette: Palette,
    /// Project, built-ins, then custom entries in persisted display order.
    pub choices: Vec<PaletteChoice>,
    /// Selected project/preset index, retained while a custom entry is selected.
    pub preset: usize,
    /// Current library and manager selection.
    pub state: PaletteState,
}

/// Capabilities available to a palette panel invocation.
pub trait PaletteCx {
    /// Displayed project palette, including a read-only history preview.
    fn palette(&self) -> &Palette;
    /// Current brush color, independent of the selected library palette.
    fn active_color(&self) -> &Swatch;
    /// Last admitted panel state. Plugins return a replacement rather than mutating it.
    fn state(&self) -> &PaletteState;
    /// Only settings declared by the panel's owner are visible.
    fn setting(&self, key: &str) -> Option<&SettingValue>;
}

/// An existing palette manager operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteOperation {
    /// Create a one-color custom palette.
    Create,
    /// Copy the selected palette.
    Duplicate,
    /// Delete a custom entry.
    Delete,
    /// Rename a custom entry.
    Rename,
    /// Replace the selected swatch.
    ApplyColor,
    /// Append a swatch.
    AddSlot,
    /// Remove the selected swatch.
    RemoveSlot,
    /// Move the selected swatch left in display order.
    MoveLeft,
    /// Move the selected swatch right in display order.
    MoveRight,
}

/// Destination for a committed color from the recent-color list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecentColorTarget {
    /// Replace the selected manager swatch, preserving its name.
    SelectedSwatch,
    /// Insert and select a swatch; None appends and overflow clamps.
    NewSwatch {
        /// Requested display position.
        at: Option<usize>,
    },
}

/// Typed input from the shell or agent adapter.
#[derive(Clone, Debug)]
pub enum PaletteAction {
    /// Decode the existing local library format; absence starts empty.
    LoadLibrary(Option<Vec<u8>>),
    /// Choose a project or preset palette without changing the brush or document.
    ChoosePreset(usize),
    /// Choose a custom entry without changing the brush or document.
    ChooseCustom(Uuid),
    /// Choose any palette by its stable address, without changing the brush.
    ChoosePalette(PaletteId),
    /// Select a manager swatch, without changing the brush.
    SelectSlot(usize),
    /// Set the selected swatch's exact opaque RGB, preserving its name.
    /// A preview creates at most one custom copy and produces no library bytes.
    /// Other actions cancel the preview first, except CommitColor and CancelColor.
    SetColor {
        /// Exact sRGB bytes, including values sampled by an eyedropper.
        rgb: [u8; 3],
        /// Persist this value and end any active preview.
        commit: bool,
    },
    /// Persist the current preview once; no-op when no preview exists.
    CommitColor,
    /// Restore the library and selection from before the current preview.
    CancelColor,
    /// Insert a swatch and select it, using the lowest free stable slot.
    AddSwatch {
        /// Exact opaque sRGB bytes.
        rgb: [u8; 3],
        /// Display position before which to insert; None appends, overflow clamps.
        at: Option<usize>,
    },
    /// Remove a swatch by display position; the final swatch cannot be removed.
    RemoveSwatch(usize),
    /// Move a swatch to a final display position, preserving all stable slots.
    MoveSwatch {
        /// Existing display position; out-of-range sources are rejected.
        from: usize,
        /// Final display position, clamped to the last position.
        to: usize,
    },
    /// Copy a swatch and its name immediately after its display position; select it.
    DuplicateSwatch(usize),
    /// Commit a recent color to the manager without changing the brush.
    UseRecentColor {
        /// Current zero-based MRU position; invalid positions are rejected.
        index: usize,
        /// Replace the selected swatch or insert a new one.
        target: RecentColorTarget,
    },
    /// Select a brush swatch by display position.
    SelectSwatch(usize),
    /// Move the brush selection, wrapping in display order.
    StepSwatch(isize),
    /// Edit the local library; the outcome must be persisted before adoption.
    Edit {
        /// Existing manager operation.
        operation: PaletteOperation,
        /// Current name field.
        name: String,
        /// Current hex field.
        color: String,
    },
    /// Import a custom library entry from bounded HEX/GPL text.
    ImportLibrary {
        /// Filename stem, truncated by the implementation as before.
        name: String,
        /// Lowercase file extension.
        format: String,
        /// File contents; reading is the host's responsibility.
        text: String,
    },
    /// Execute one of the owner's declared shell commands.
    Command(String),
    /// Propose a typed core palette edit.
    EditDocument(PaletteEdit),
    /// Set an existing document slot or add it if absent.
    SetSlot {
        /// Stable nonzero document slot.
        slot: u16,
        /// Original opaque color and optional name.
        color: Swatch,
    },
    /// Append an exact built-in preset to the document.
    AppendPreset(String),
    /// Append imported colors without recoloring existing geometry.
    AppendImport {
        /// Lowercase HEX/GPL format.
        format: String,
        /// Bounded source text.
        text: String,
    },
}

/// A shell presentation request from a palette command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PalettePresentation {
    /// Focus the registered swatch panel.
    Focus,
    /// Open the palette chooser.
    Choose,
    /// Open the library manager.
    Manage,
}

/// Proposed state and typed edits; only the host can admit these effects.
#[derive(Clone, Debug, Default)]
pub struct PaletteOutcome {
    /// Source name when this interaction automatically copied a read-only palette.
    pub copied_from: Option<String>,
    /// Next panel state, installed after successful persistence/edit admission.
    pub state: PaletteState,
    /// Replacement library bytes in the existing settings format, if changed.
    pub library_bytes: Option<Vec<u8>>,
    /// A proposed owner-scoped setting change.
    pub setting: Option<(String, SettingValue)>,
    /// New brush color, if selected.
    pub color: Option<Swatch>,
    /// Brush display position, for shell scrolling.
    pub selected: Option<usize>,
    /// Shell focus or modal request.
    pub presentation: Option<PalettePresentation>,
    /// Ordered typed core commands. No direct document mutation is available.
    pub commands: Vec<Command>,
    /// Source-to-document slots for an append operation.
    pub slot_mapping: Vec<(u16, u16)>,
}

/// A validation or interaction failure that leaves host state unchanged.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PaletteError {
    /// An append cannot fit without replacing existing document colors.
    #[error("not enough free palette slots; imports append without recoloring existing geometry")]
    Capacity,
    /// Core palette validation failed.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// Existing user-facing library or input error.
    #[error("{0}")]
    Message(String),
}

/// Portable behavior bound to a palette panel declaration.
pub trait PalettePanel {
    /// Describe the panel using only its scoped capabilities.
    fn view(&self, cx: &dyn PaletteCx) -> PaletteView;
    /// Propose an atomic interaction outcome; the host owns adoption and persistence.
    fn interact(
        &self,
        cx: &dyn PaletteCx,
        action: PaletteAction,
    ) -> Result<PaletteOutcome, PaletteError>;
}
