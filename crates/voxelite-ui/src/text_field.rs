//! GPUI input, clipboard, focus and painting adapter for the pure editing model.
use crate::{
    Dialog, FieldFrame, Theme,
    editing::{
        self, ClipboardCommand, Command, EditKind, Editor, EscapePolicy, NumericPolicy, Unit,
    },
    tokens::{self, metrics, radius, space, type_size},
};
use gpui::{prelude::*, *};
use std::{ops::Range, time::Duration};

/// Field-local action. Its binding context prevents document undo interception.
#[derive(Clone, Debug, PartialEq, Eq, Action)]
#[action(no_json)]
pub struct TextFieldAction {
    /// Pure command to execute.
    pub command: Command,
}
/// Native menu actions use the same clipboard implementation as key bindings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Action)]
#[action(no_json)]
pub enum ClipboardOp {
    /// Remove selected text after copying it.
    Cut,
    /// Copy selected text.
    Copy,
    /// Insert clipboard text.
    Paste,
    /// Select the whole field.
    SelectAll,
}
impl From<ClipboardOp> for ClipboardCommand {
    fn from(value: ClipboardOp) -> Self {
        match value {
            ClipboardOp::Cut => Self::Cut,
            ClipboardOp::Copy => Self::Copy,
            ClipboardOp::Paste => Self::Paste,
            ClipboardOp::SelectAll => Self::SelectAll,
        }
    }
}
struct Installed;
impl Global for Installed {}
/// Install field bindings once per application. Construction also calls this.
pub fn install_text_field_bindings(cx: &mut App) {
    install_text_field_bindings_for_platform(cfg!(target_os = "macos"), cx);
}
/// Install bindings for the host OS. Browser shells should call this before
/// creating fields, since a wasm target cannot identify the browser's host OS.
pub fn install_text_field_bindings_for_platform(mac: bool, cx: &mut App) {
    if cx.has_global::<Installed>() {
        return;
    }
    cx.set_global(Installed);
    cx.bind_keys(key_bindings(mac));
}
fn key_bindings(mac: bool) -> Vec<KeyBinding> {
    editing::bindings(mac)
        .map(|b| {
            KeyBinding::new(
                b.key,
                TextFieldAction { command: b.command },
                Some("TextField"),
            )
        })
        .collect()
}
/// Events emitted by every shared field; app adapters own domain effects.
#[derive(Clone, Debug)]
pub enum TextFieldEvent {
    /// The draft changed through typing, paste, history, cancellation or stepping.
    Changed,
    /// Enter committed the draft (also emitted for valid numeric blur).
    Committed,
    /// The field lost focus.
    Blurred,
}
/// A single-line native editor with selection, IME, clipboard and local history.
pub struct TextField {
    editor: Editor,
    focus: FocusHandle,
    placeholder: SharedString,
    theme: Theme,
    embedded: bool,
    cells: Option<usize>,
    escape_policy: EscapePolicy,
    validate_commit: fn(&str) -> bool,
    focus_scope: Option<FocusHandle>,
    return_focus: Option<FocusHandle>,
    scroll_x: Pixels,
    layout: Option<ShapedLine>,
    bounds: Bounds<Pixels>,
    drag: Option<Drag>,
    pointer_focus: bool,
    has_focus: bool,
    menu: Option<Point<Pixels>>,
    menu_index: usize,
    caret_visible: bool,
    last_activity: u64,
    clock: Box<dyn Fn() -> u64>,
    blink: Option<Task<()>>,
    autoscroll: Option<Task<()>>,
    subscriptions: Vec<Subscription>,
}
#[derive(Clone)]
enum Drag {
    Characters(usize),
    Words(Range<usize>),
    Scrub { x: Pixels, initial: f64 },
}
impl EventEmitter<TextFieldEvent> for TextField {}
impl TextField {
    /// Create a field; the placeholder also supplies its accessible label.
    pub fn new(placeholder: impl Into<SharedString>, cx: &mut App) -> Entity<Self> {
        install_text_field_bindings(cx);
        let placeholder = placeholder.into();
        // Use GPUI's monotonic clock on native and web without a second runtime
        // dependency. Capturing it also lets the adapter use GPUI's test clock.
        let executor = cx.background_executor().clone();
        let start = executor.now();
        cx.new(|cx| Self {
            editor: Editor::default(),
            // GPUI uses the tracked handle's tab policy, not the Div's flags.
            focus: cx.focus_handle().tab_stop(true),
            placeholder,
            theme: Theme::Dark,
            embedded: false,
            cells: None,
            escape_policy: EscapePolicy::Blur,
            validate_commit: |_| true,
            focus_scope: None,
            return_focus: None,
            scroll_x: px(0.),
            layout: None,
            bounds: Bounds::default(),
            drag: None,
            pointer_focus: false,
            has_focus: false,
            menu: None,
            menu_index: 0,
            caret_visible: true,
            last_activity: 0,
            clock: Box::new(move || executor.now().duration_since(start).as_millis() as u64),
            blink: None,
            autoscroll: None,
            subscriptions: Vec::new(),
        })
    }
    /// Current draft text.
    pub fn text(&self) -> &str {
        self.editor.text()
    }
    /// Update the placeholder and accessible label for a reused search field.
    pub fn placeholder(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        let label = label.into();
        if self.placeholder != label {
            self.placeholder = label;
            cx.notify();
        }
    }
    /// Replace the model value and clear local history.
    pub fn set(&mut self, text: String, cx: &mut Context<Self>) {
        self.editor.set(&text);
        self.changed(cx);
    }
    /// Set the token theme.
    pub fn appearance(&mut self, theme: Theme, cx: &mut Context<Self>) {
        if self.theme != theme {
            self.theme = theme;
            cx.notify();
        }
    }
    /// Let a search row provide the frame.
    pub fn embed(&mut self) {
        self.embedded = true;
    }
    /// Fixed-width monospaced presentation, including nonnumeric hex fields.
    pub fn numeric(&mut self, cells: usize) {
        self.cells = Some(cells);
    }
    /// Enable numeric validation, stepping and Option-drag scrubbing.
    pub fn number(&mut self, policy: NumericPolicy, value: f64, cx: &mut Context<Self>) {
        self.editor.set_numeric(policy, value);
        self.cells = Some(5);
        self.escape_policy = EscapePolicy::Revert;
        self.changed(cx);
    }
    /// Explicit per-field cancellation behavior.
    pub fn escape_policy(&mut self, policy: EscapePolicy) {
        self.escape_policy = policy;
    }
    /// Keep an invalid domain draft focused without changing its committed
    /// baseline. Numeric validation is already provided by `NumericPolicy`.
    pub fn validate_commit(&mut self, validate: fn(&str) -> bool) {
        self.validate_commit = validate;
    }
    /// Optional modal tab scope and focus destination after Escape.
    pub fn focus_policy(&mut self, scope: Option<FocusHandle>, return_focus: FocusHandle) {
        self.focus_scope = scope;
        self.return_focus = Some(return_focus);
    }
    /// Whether the field currently owns keyboard input.
    pub fn focused(&self, window: &Window) -> bool {
        self.focus.is_focused(window)
    }
    /// Focus from a keyboard command, selecting all on entry.
    pub fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.select_all();
        window.focus(&self.focus, cx);
        self.activity(cx);
    }
    /// Establish a committed baseline without erasing local history.
    pub fn commit(&mut self, cx: &mut Context<Self>) {
        self.editor.commit();
        self.changed(cx);
    }
    fn time(&self) -> u64 {
        (self.clock)()
    }
    fn activity(&mut self, cx: &mut Context<Self>) {
        self.last_activity = self.time();
        self.caret_visible = true;
        cx.notify();
    }
    fn changed(&mut self, cx: &mut Context<Self>) {
        self.activity(cx);
        cx.emit(TextFieldEvent::Changed);
    }
    fn blur(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(focus) = &self.return_focus {
            window.focus(focus, cx);
        } else {
            window.blur(cx);
        }
    }
    fn clipboard(&mut self, command: ClipboardCommand, cx: &mut Context<Self>) {
        let before = self.text().to_owned();
        self.editor.break_run();
        match command {
            ClipboardCommand::SelectAll => self.editor.select_all(),
            ClipboardCommand::Cut | ClipboardCommand::Copy => {
                let selection = self.editor.selection();
                if !selection.is_empty() {
                    cx.write_to_clipboard(ClipboardItem::new_string(self.text()[selection].into()));
                    if command == ClipboardCommand::Cut {
                        self.editor.delete(Unit::Grapheme, false, self.time());
                    }
                }
            }
            ClipboardCommand::Paste => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.editor
                        .replace(None, &text, EditKind::Atomic, self.time());
                }
            }
        }
        self.activity(cx);
        if self.text() != before {
            cx.emit(TextFieldEvent::Changed);
        }
    }
    fn command(&mut self, command: Command, window: &mut Window, cx: &mut Context<Self>) {
        let before = self.text().to_owned();
        if self.menu.is_some() {
            match command {
                Command::Escape => self.menu = None,
                Command::Vertical(up, _, _) => {
                    self.menu_index = if up {
                        (self.menu_index + 3) % 4
                    } else {
                        (self.menu_index + 1) % 4
                    }
                }
                Command::Commit => {
                    let op = Self::menu_commands()[self.menu_index].1;
                    if self.menu_enabled(op, cx) {
                        self.clipboard(op, cx);
                    }
                    self.menu = None;
                }
                _ => self.menu = None,
            }
            self.activity(cx);
            cx.stop_propagation();
            return;
        }
        match command {
            Command::Move(..)
            | Command::Delete(..)
            | Command::Vertical(..)
            | Command::Undo
            | Command::Redo => {
                self.editor.apply(command, self.time());
            }
            Command::Clipboard(op) => {
                self.clipboard(op, cx);
                cx.stop_propagation();
                return;
            }
            Command::Commit => {
                if self.editor.commit_if_valid(self.validate_commit) {
                    cx.emit(TextFieldEvent::Committed);
                }
            }
            Command::Escape => {
                self.editor.escape(self.escape_policy);
                self.blur(window, cx);
            }
            Command::Tab(backwards) => {
                self.editor.break_run();
                if let Some(scope) = &self.focus_scope {
                    Dialog::trap_tab(scope, backwards, window, cx);
                } else if backwards {
                    window.focus_prev(cx);
                } else {
                    window.focus_next(cx);
                }
            }
            Command::Menu => {
                self.menu = Some(self.bounds.bottom_left());
                self.menu_index = 0;
            }
        }
        // Always consume undo, including an empty stack. Document history is
        // never a fallback while a field owns focus.
        cx.stop_propagation();
        self.activity(cx);
        if self.text() != before {
            cx.emit(TextFieldEvent::Changed);
        }
    }
    fn install_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.subscriptions.is_empty() {
            return;
        }
        self.subscriptions
            .push(cx.on_focus(&self.focus, window, |this, window, cx| {
                if !this.has_focus && !this.pointer_focus {
                    this.editor.select_all();
                }
                this.has_focus = true;
                this.pointer_focus = false;
                this.activity(cx);
                this.start_blink(window, cx);
            }));
        self.subscriptions
            .push(cx.on_blur(&self.focus, window, |this, window, cx| {
                this.blink = None;
                this.autoscroll = None;
                this.drag = None;
                this.menu = None;
                // GPUI also reports window deactivation as blur. Preserve the
                // draft and selection until focus actually leaves this field.
                if this.focused(window) {
                    this.editor.break_run();
                    cx.notify();
                    return;
                }
                this.has_focus = false;
                if this.editor.numeric().is_some() {
                    this.editor.commit();
                    cx.emit(TextFieldEvent::Committed);
                } else {
                    this.editor.unmark();
                }
                cx.emit(TextFieldEvent::Blurred);
                this.changed(cx);
            }));
        self.subscriptions
            .push(cx.observe_window_activation(window, |this, window, cx| {
                this.blink = None;
                this.autoscroll = None;
                this.drag = None;
                this.editor.break_run();
                if window.is_window_active() && this.focused(window) {
                    this.activity(cx);
                    this.start_blink(window, cx);
                }
                cx.notify();
            }));
        if self.focused(window) {
            self.start_blink(window, cx);
        }
    }
    fn start_blink(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.blink.is_some() {
            return;
        }
        self.blink = Some(cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                let keep = this
                    .update_in(cx, |this, window, cx| {
                        if !this.focused(window) || !window.is_window_active() {
                            return false;
                        }
                        if this.time().saturating_sub(this.last_activity) >= 500 {
                            this.caret_visible = !this.caret_visible;
                            cx.notify();
                        }
                        true
                    })
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
        }));
    }
    fn index_at(&self, x: Pixels) -> usize {
        self.layout
            .as_ref()
            .map_or(self.text().len(), |line| {
                line.closest_index_for_x(x - self.bounds.left() + self.scroll_x)
            })
            .min(self.text().len())
    }
    fn drag_to(&mut self, x: Pixels, coarse: bool, cx: &mut Context<Self>) {
        let before = self.text().to_owned();
        match self.drag.clone() {
            Some(Drag::Characters(anchor)) => self.editor.select(anchor, self.index_at(x)),
            Some(Drag::Words(word)) => self.editor.select_words(word, self.index_at(x)),
            Some(Drag::Scrub { x: start, initial }) => self.editor.scrub(
                initial,
                f64::from(f32::from(x - start)),
                coarse,
                self.time(),
            ),
            None => return,
        }
        self.activity(cx);
        if self.text() != before {
            cx.emit(TextFieldEvent::Changed);
        }
    }
    fn start_autoscroll(&mut self, window: &Window, cx: &mut Context<Self>) {
        self.autoscroll = Some(cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(30))
                    .await;
                let keep = this
                    .update_in(cx, |this, window, cx| {
                        if this.drag.is_none()
                            || !this.focused(window)
                            || !window.is_window_active()
                        {
                            return false;
                        }
                        // Element mouse-move handlers only fire while hovered.
                        // Sample the window pointer so stationary outside drags
                        // keep selecting and scrolling until mouse-up.
                        let x = window.mouse_position().x;
                        if matches!(this.drag, Some(Drag::Scrub { .. })) {
                            this.drag_to(x, window.modifiers().shift, cx);
                            return true;
                        }
                        let delta = if x < this.bounds.left() {
                            x - this.bounds.left()
                        } else if x > this.bounds.right() {
                            x - this.bounds.right()
                        } else {
                            px(0.)
                        };
                        if delta != px(0.) {
                            let max = this.layout.as_ref().map_or(px(0.), |line| {
                                (line.width - this.bounds.size.width).max(px(0.))
                            });
                            this.scroll_x = (this.scroll_x + delta * 0.25).clamp(px(0.), max);
                            this.drag_to(x, false, cx);
                        }
                        true
                    })
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
        }));
    }
    fn mouse_down(&mut self, e: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.pointer_focus = !self.focused(window);
        window.focus(&self.focus, cx);
        self.menu = None;
        self.editor.unmark();
        self.editor.break_run();
        if e.modifiers.alt
            && let Some(policy) = self.editor.numeric()
        {
            self.drag = Some(Drag::Scrub {
                x: e.position.x,
                initial: policy.parse(self.text()).unwrap_or(policy.min),
            });
        } else {
            let p = self.index_at(e.position.x);
            self.drag = match e.click_count {
                3.. => {
                    self.editor.select_all();
                    None
                }
                2 => {
                    let word = self.editor.word_at(p);
                    self.editor.select(word.start, word.end);
                    Some(Drag::Words(word))
                }
                _ => {
                    let anchor = if e.modifiers.shift {
                        self.editor.anchor()
                    } else {
                        p
                    };
                    self.editor.select(anchor, p);
                    Some(Drag::Characters(anchor))
                }
            };
        }
        self.start_autoscroll(window, cx);
        self.activity(cx);
        cx.stop_propagation();
    }
    fn end_drag(&mut self, cx: &mut Context<Self>) {
        self.drag = None;
        self.autoscroll = None;
        self.editor.break_run();
        self.activity(cx);
    }
    fn menu_commands() -> [(&'static str, ClipboardCommand); 4] {
        [
            ("Cut", ClipboardCommand::Cut),
            ("Copy", ClipboardCommand::Copy),
            ("Paste", ClipboardCommand::Paste),
            ("Select All", ClipboardCommand::SelectAll),
        ]
    }
    fn menu_enabled(&self, op: ClipboardCommand, cx: &mut App) -> bool {
        match op {
            ClipboardCommand::Cut | ClipboardCommand::Copy => !self.editor.selection().is_empty(),
            ClipboardCommand::Paste => cx
                .read_from_clipboard()
                .and_then(|item| item.text())
                .is_some_and(|s| !s.is_empty()),
            ClipboardCommand::SelectAll => !self.text().is_empty(),
        }
    }
    fn context_menu(&self, position: Point<Pixels>, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        deferred(
            anchored().position(position).snap_to_window().child(
                div()
                    .id("field-menu")
                    .role(Role::Menu)
                    .occlude()
                    .cursor_default()
                    .w(px(180.))
                    .p(px(space::XS))
                    .rounded(px(radius::MD))
                    .border_1()
                    .border_color(t.separator())
                    .bg(t.surface_raised())
                    .shadow(tokens::panel_shadow())
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                        this.menu = None;
                        this.blur(window, cx);
                        cx.notify();
                    }))
                    .children(Self::menu_commands().into_iter().enumerate().map(
                        |(i, (label, op))| {
                            let enabled = self.menu_enabled(op, cx);
                            div()
                                .id(("field-menu-item", i))
                                .role(Role::MenuItem)
                                .aria_label(label)
                                .when(!enabled, |d| d.aria_description("Unavailable"))
                                .px(px(space::LG))
                                .py(px(space::MD))
                                .rounded(px(radius::SM))
                                .text_color(if enabled { t.text() } else { t.muted() })
                                .child(label)
                                .when(i == self.menu_index, |d| d.bg(t.hover()))
                                .when(enabled, |d| {
                                    d.cursor_pointer()
                                        .hover(move |d| d.bg(t.hover()))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.clipboard(op, cx);
                                                this.menu = None;
                                                cx.stop_propagation();
                                            }),
                                        )
                                })
                        },
                    )),
            ),
        )
        .with_priority(10)
    }
}
impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        r: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = editing::from_utf16(self.text(), r);
        *actual = Some(editing::to_utf16(self.text(), r.clone()));
        Some(self.text()[r].into())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: editing::to_utf16(self.text(), self.editor.selection()),
            reversed: self.editor.reversed(),
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.editor
            .marked()
            .map(|r| editing::to_utf16(self.text(), r))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.editor.unmark();
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.menu = None;
        let r = r.map(|r| editing::from_utf16(self.text(), r));
        let kind = if self.editor.marked().is_some() {
            EditKind::Composition
        } else {
            EditKind::Typing
        };
        self.editor.replace(r, text, kind, self.time());
        if kind == EditKind::Composition {
            self.editor.unmark();
        }
        self.changed(cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = r.map(|r| editing::from_utf16(self.text(), r));
        self.editor.compose(r, text, selected, self.time());
        self.changed(cx);
    }
    fn bounds_for_range(
        &mut self,
        r: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let r = editing::from_utf16(self.text(), r);
        let line = self.layout.as_ref()?;
        Some(Bounds::from_corners(
            point(
                self.bounds.left() - self.scroll_x + line.x_for_index(r.start),
                self.bounds.top(),
            ),
            point(
                self.bounds.left() - self.scroll_x + line.x_for_index(r.end),
                self.bounds.bottom(),
            ),
        ))
    }
    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let i = self.index_at(p.x);
        Some(editing::to_utf16(self.text(), i..i).start)
    }
}
impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.install_window(window, cx);
        let t = self.theme;
        let entity = cx.entity();
        let focus = self.focus.clone();
        let focused = self.focused(window);
        let mut frame = FieldFrame::new(t).focused(focused).embedded(self.embedded);
        if let Some(cells) = self.cells {
            frame = frame.numeric(cells);
        }
        let menu = self
            .menu
            .map(|p| self.context_menu(p, cx).into_any_element());
        frame
            .build()
            .id("text-field")
            .role(Role::TextInput)
            .aria_label(self.placeholder.clone())
            .aria_placeholder(self.placeholder.clone())
            .aria_value(self.text().to_owned())
            .tab_index(0)
            .track_focus(&self.focus)
            .key_context("TextField SharedTextInput")
            .when(focused && !self.embedded, |d| {
                d.shadow(tokens::focus_ring(t))
            })
            .when(matches!(self.drag, Some(Drag::Scrub { .. })), |d| {
                d.cursor(CursorStyle::ResizeLeftRight)
            })
            .on_action(cx.listener(|this, action: &TextFieldAction, window, cx| {
                this.command(action.command, window, cx)
            }))
            .on_action(cx.listener(|this, op: &ClipboardOp, _, cx| {
                this.clipboard((*op).into(), cx);
                cx.stop_propagation();
            }))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    this.pointer_focus = !this.focused(window);
                    window.focus(&this.focus, cx);
                    if this.editor.selection().is_empty() {
                        let p = this.index_at(e.position.x);
                        this.editor.select(p, p);
                    }
                    this.menu = Some(e.position);
                    this.menu_index = 0;
                    this.activity(cx);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_down_out(cx.listener(|this, _, window, cx| {
                if this.focused(window) && this.menu.is_none() {
                    this.blur(window, cx);
                }
            }))
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                if e.pressed_button == Some(MouseButton::Left) && this.drag.is_some() {
                    this.drag_to(e.position.x, e.modifiers.shift, cx);
                    cx.stop_propagation();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.end_drag(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.end_drag(cx)),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        window.handle_input(
                            &focus,
                            ElementInputHandler::new(bounds, entity.clone()),
                            cx,
                        );
                        entity.update(cx, |this, cx| {
                            let content: SharedString = if this.text().is_empty() {
                                this.placeholder.clone()
                            } else {
                                this.text().to_owned().into()
                            };
                            let style = window.text_style();
                            let color = if this.text().is_empty() {
                                t.muted()
                            } else {
                                t.text()
                            };
                            let line = window.text_system().shape_line(
                                content.clone(),
                                px(type_size::BODY_LG),
                                &[TextRun {
                                    len: content.len(),
                                    font: style.font(),
                                    color,
                                    background_color: None,
                                    underline: None,
                                    strikethrough: None,
                                }],
                                None,
                            );
                            let caret_x = line.x_for_index(this.editor.head());
                            if focused && this.drag.is_none() {
                                let width =
                                    (bounds.size.width - px(space::XXS)).max(px(metrics::EDGE));
                                this.scroll_x =
                                    this.scroll_x.min(caret_x).max(caret_x - width).max(px(0.));
                            }
                            if this.text().is_empty() {
                                this.scroll_x = px(0.);
                            }
                            let origin = point(bounds.left() - this.scroll_x, bounds.top());
                            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                                if focused && window.is_window_active() {
                                    let selection = this.editor.selection();
                                    let a = line.x_for_index(selection.start);
                                    let b = line.x_for_index(selection.end);
                                    if a != b || this.caret_visible {
                                        window.paint_quad(fill(
                                            Bounds::new(
                                                point(origin.x + a, bounds.top()),
                                                size(
                                                    if a == b { px(metrics::EDGE) } else { b - a },
                                                    bounds.size.height,
                                                ),
                                            ),
                                            if a == b {
                                                t.accent()
                                            } else {
                                                t.text_selection()
                                            },
                                        ));
                                    }
                                }
                                let _ = line.paint(
                                    origin,
                                    px(type_size::LINE),
                                    TextAlign::Left,
                                    None,
                                    window,
                                    cx,
                                );
                                if let Some(marked) = this.editor.marked() {
                                    let a = line.x_for_index(marked.start);
                                    let b = line.x_for_index(marked.end);
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(origin.x + a, bounds.bottom() - px(1.)),
                                            size(b - a, px(1.)),
                                        ),
                                        t.text(),
                                    ));
                                }
                            });
                            this.layout = Some(line);
                            this.bounds = bounds;
                        });
                    },
                )
                .size_full(),
            )
            .children(menu)
    }
}

#[cfg(test)]
mod tests {
    use super::{ClipboardOp, Command, TextFieldAction, editing, key_bindings};
    use gpui::{KeyBinding, KeyContext, Keymap, Keystroke};

    #[test]
    fn installed_bindings_parse_and_only_resolve_in_a_field() {
        for mac in [true, false] {
            let bindings = key_bindings(mac);
            let keymap = Keymap::new(bindings.clone());
            for (binding, expected) in bindings.iter().zip(editing::bindings(mac)) {
                for (context, count) in [("TextField SharedTextInput", 1), ("Editor", 0)] {
                    let (matched, pending) = keymap.bindings_for_input(
                        binding.keystrokes(),
                        &[KeyContext::parse(context).unwrap()],
                    );
                    assert!(!pending, "{}", expected.key);
                    assert_eq!(matched.len(), count, "{} in {context}", expected.key);
                    if let Some(matched) = matched.first() {
                        assert_eq!(
                            matched.action().as_any().downcast_ref::<TextFieldAction>(),
                            Some(&TextFieldAction {
                                command: expected.command
                            }),
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn field_undo_bindings_take_precedence_over_document_bindings() {
        for mac in [true, false] {
            let primary = if mac { "cmd-z" } else { "ctrl-z" };
            // A distinct action represents a document's global Undo binding.
            let mut bindings = vec![KeyBinding::new(primary, ClipboardOp::Cut, None)];
            bindings.extend(key_bindings(mac));
            let keymap = Keymap::new(bindings);
            let (matched, _) = keymap.bindings_for_input(
                &[Keystroke::parse(primary).unwrap()],
                &[
                    KeyContext::parse("Editor").unwrap(),
                    KeyContext::parse("TextField").unwrap(),
                ],
            );
            assert_eq!(
                matched[0]
                    .action()
                    .as_any()
                    .downcast_ref::<TextFieldAction>(),
                Some(&TextFieldAction {
                    command: Command::Undo
                }),
            );
        }
    }
}
