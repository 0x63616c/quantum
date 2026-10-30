//! Portable overlay drawing and pointer feedback, without renderer or GUI access.
use crate::{Command, Region, RenderPassSlot, SettingValue, ToolError};

/// Semantic colors resolved by the shell's current theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayColor {
    /// World X, Y or Z color (indices 0, 1, 2).
    Axis(usize),
    /// Subtle spatial reference color.
    Line,
    /// Window background.
    Background,
    /// Foreground text/highlight.
    Text,
}

/// A finite square lattice on a world-space horizontal plane.
/// Kept analytic so GPU backends can preserve their antialiased grid resolve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayGrid {
    /// Integer height, kept separate from the offset for backend float precision.
    pub plane: i32,
    /// World-space offset below or above that plane.
    pub offset: f64,
    /// Inclusive minimum X and Z coordinate.
    pub min: i32,
    /// Inclusive maximum X and Z coordinate.
    pub max: i32,
    /// Positive integer spacing in world units.
    pub spacing: u32,
    /// Logical pixel stroke width for path backends.
    pub width: f32,
    /// Theme color.
    pub color: OverlayColor,
    /// Color opacity.
    pub opacity: f32,
}

/// One line in a widget's local logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayLine {
    /// Starting position.
    pub from: [f32; 2],
    /// Ending position.
    pub to: [f32; 2],
    /// Stroke width.
    pub width: f32,
    /// Theme color.
    pub color: OverlayColor,
}

/// A circular widget target, painted in declaration order.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayCircle {
    /// Center in local logical pixels.
    pub center: [f32; 2],
    /// Radius in logical pixels.
    pub radius: f32,
    /// Fill color.
    pub fill: OverlayColor,
    /// Border color (two logical pixels).
    pub border: OverlayColor,
    /// Optional centered, semibold 12px label.
    pub label: Option<String>,
    /// Label color.
    pub text: OverlayColor,
    /// Whether the host should paint its standard focus ring.
    pub highlighted: bool,
}

/// An interactive overlay anchored at the viewport's top-right corner.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayWidget {
    /// Square side in logical pixels.
    pub size: f32,
    /// Top and right inset in logical pixels.
    pub inset: f32,
    /// Accessible description.
    pub label: String,
    /// Lines painted behind all circles, in order.
    pub lines: Vec<OverlayLine>,
    /// Circles painted back to front.
    pub circles: Vec<OverlayCircle>,
}

/// Renderer-independent drawing emitted by an overlay.
#[derive(Clone, Debug, PartialEq)]
pub enum OverlayPrimitive {
    /// World-space grid, composited behind the document.
    Grid(OverlayGrid),
    /// Screen-space interactive widget above the document.
    Widget(OverlayWidget),
    /// A projected discussion pin, drawn above the document.
    Pin(OverlayPin),
}

/// A projected annotation with a shell-rendered hit target.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayPin {
    /// Thread selected when the target is clicked.
    pub id: crate::Uuid,
    /// Original document index, preserving overlap and element order.
    pub index: usize,
    /// Normalized viewport position of the anchor center.
    pub uv: [f64; 2],
    /// Stable display number.
    pub number: usize,
    /// Whether the thread is resolved.
    pub resolved: bool,
    /// Whether the pin is selected.
    pub selected: bool,
    /// Whether the anchor is occluded, hidden or orphaned.
    pub dim: bool,
    /// Plain-text hover description.
    pub tooltip: String,
}

/// One enabled contribution's output; IDs route later interaction through the host.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayFrame {
    /// Declared render-pass ID.
    pub id: String,
    /// Requested insertion slot.
    pub slot: RenderPassSlot,
    /// Draw primitives in emission order.
    pub primitives: Vec<OverlayPrimitive>,
}

/// Scoped overlay capabilities. No engine, app, filesystem or GUI is exposed.
pub trait OverlayCx {
    /// Bounds of the displayed document, including history previews.
    fn bounds(&self) -> Region;
    /// Display camera's right, up and back unit vectors in world space.
    fn camera_basis(&self) -> [[f64; 3]; 3];
    /// Only this overlay owner's declared settings are readable.
    fn setting(&self, key: &str) -> Option<&SettingValue>;
    /// Optional displayed discussion context for annotation overlays.
    fn comments(&self) -> Option<&dyn crate::CommentsCx> {
        None
    }
    /// Project a world point through the host camera; invalid viewports return none.
    fn project(&self, _point: [f64; 3]) -> Option<[f64; 2]> {
        None
    }
    /// Nearest visible voxel at a normalized viewport position.
    fn pick(&self, _uv: [f64; 2]) -> Option<crate::Pos> {
        None
    }
    /// Append a portable primitive; partial output is discarded if drawing panics.
    fn emit(&mut self, primitive: OverlayPrimitive);
}

/// Host-retained transient pointer state, never part of document history.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayDrag {
    /// Initial pointer position in widget-local logical pixels.
    pub origin: [f32; 2],
    /// Previous pointer position.
    pub last: [f32; 2],
    /// Plugin-defined pressed target identity.
    pub target: Option<u32>,
    /// Whether the plugin's drag threshold has been crossed.
    pub moved: bool,
}

/// Pointer events independent of the GUI framework.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayGesture {
    /// Query the hovered target without navigation or edits.
    Hover,
    /// Begin a primary-button gesture.
    Press {
        /// Whether this is a double click.
        double: bool,
    },
    /// Advance or cancel a gesture according to button state.
    Move {
        /// Whether the primary button remains held.
        primary_down: bool,
    },
    /// Finish a gesture, possibly snapping to its pressed target.
    Release,
}

/// One event and the last successful gesture state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayInput {
    /// Kind of pointer event.
    pub gesture: OverlayGesture,
    /// Position relative to the widget, including positions outside its bounds.
    pub position: [f32; 2],
    /// Window-space logical pointer position, used for drag deltas even if layout moves.
    pub pointer: [f32; 2],
    /// Previous transient state, absent after cancellation or disablement.
    pub drag: Option<OverlayDrag>,
}

/// Navigation requests interpreted by the core camera; never document edits.
#[derive(Clone, Debug, PartialEq)]
pub enum OverlayNavigation {
    /// Orbit by incremental logical pointer pixels.
    Orbit([f64; 2]),
    /// Orient without changing target or zoom.
    Orient {
        /// Azimuth in radians.
        yaw: f64,
        /// Elevation in radians.
        pitch: f64,
        /// Optional existing shell command ID to mark as the selected view.
        view: Option<String>,
    },
    /// Invoke the core's existing isometric fit action.
    Frame,
}

/// Proposed edits and transient feedback. Only the host admits commands to undo.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverlayOutcome {
    /// Ordered typed core commands; empty for camera navigation.
    pub commands: Vec<Command>,
    /// Transaction label when commands are present.
    pub label: String,
    /// Next transient gesture state.
    pub drag: Option<OverlayDrag>,
    /// Plugin-defined hovered target identity.
    pub hovered: Option<u32>,
    /// Optional navigation request, separate from document commands.
    pub navigation: Option<OverlayNavigation>,
}

/// Behavior bound to an existing render-pass contribution through [`crate::Plugin`].
pub trait Overlay {
    /// Emit drawing primitives using only scoped read capabilities.
    fn draw(&self, cx: &mut dyn OverlayCx, hovered: Option<u32>);
    /// Resolve interaction without mutating the document or camera.
    fn interact(
        &self,
        _cx: &dyn OverlayCx,
        _input: OverlayInput,
    ) -> Result<OverlayOutcome, ToolError> {
        Ok(OverlayOutcome::default())
    }
}
