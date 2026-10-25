//! Native GPUI shell. Portable editing logic lives in the library.
use crate::{
    actions::{MENUS, Op},
    theme::Theme,
};
use glam::DVec3;
use gpui::{prelude::*, *};
use std::{cell::Cell, path::PathBuf, rc::Rc, sync::Arc};
use voxelite_app::editor::{Editor, EditorResult, OrbitCamera, Tool};
use voxelite_core::{Palette, Pos, Preset};
use voxelite_render::{RenderOptions, View};

pub struct EditorView {
    editor: Editor,
    theme: Theme,
    focus: FocusHandle,
    image: Option<Arc<RenderImage>>,
    rendering: bool,
    render_pending: bool,
    render_size: [u32; 2],
    display_camera: OrbitCamera,
    navigation_epoch: u64,
    interactive_render: bool,
    viewport: Rc<Cell<Bounds<Pixels>>>,
    hover: Option<Pos>,
    cursor: Pos,
    drag: Option<Point<Pixels>>,
    grid: bool,
    menu: Option<usize>,
    menu_index: usize,
    palette_open: bool,
    palette_index: usize,
    command_open: bool,
    query: String,
    command_index: usize,
    command_scroll: ScrollHandle,
    dialog_open: bool,
    info: Option<(&'static str, &'static str)>,
    pending: Option<Op>,
    status: String,
    error: bool,
}
fn icon(name: &'static str) -> Svg {
    svg()
        .path(format!("icons/{name}.svg"))
        .size(px(17.0))
        .flex_shrink_0()
}
struct Hint {
    text: SharedString,
    theme: Theme,
}
impl Render for Hint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.0))
            .py(px(7.0))
            .rounded(px(6.0))
            .border_1()
            .border_color(self.theme.line())
            .bg(self.theme.background())
            .text_color(self.theme.text())
            .font_family("Geist")
            .text_size(px(12.0))
            .shadow_lg()
            .child(self.text.clone())
    }
}
fn row() -> Div {
    div().flex().items_center()
}
fn mono(text: impl Into<SharedString>) -> Div {
    div()
        .font_family("Geist Mono")
        .text_size(px(12.0))
        .child(text.into())
}
const PALETTES: [(&str, Option<Preset>); 8] = [
    ("Project colors", None),
    ("PICO-8", Some(Preset::Pico8)),
    ("ENDESGA 32", Some(Preset::Endesga32)),
    ("Sweetie 16", Some(Preset::Sweetie16)),
    ("Grayscale", Some(Preset::Grayscale)),
    ("Neutral", Some(Preset::Neutral)),
    ("Warm", Some(Preset::Warm)),
    ("Vivid", Some(Preset::Vivid)),
];

impl EditorView {
    pub fn new(editor: Editor, theme: Theme, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let weak = cx.weak_entity();
        window.on_window_should_close(cx, move |_, cx| {
            weak.update(cx, |this, cx| {
                if this.editor.engine.is_dirty() {
                    this.pending = Some(Op::Quit);
                    cx.notify();
                    false
                } else {
                    true
                }
            })
            .unwrap_or(true)
        });
        Self {
            cursor: editor.snapshot().bounds.min,
            display_camera: editor.camera,
            navigation_epoch: 0,
            interactive_render: false,
            editor,
            theme,
            focus,
            image: None,
            rendering: false,
            render_pending: false,
            render_size: [0, 0],
            viewport: Rc::new(Cell::new(Bounds::default())),
            hover: None,
            drag: None,
            grid: true,
            menu: None,
            menu_index: 0,
            palette_open: false,
            palette_index: 0,
            command_open: false,
            query: String::new(),
            command_index: 0,
            command_scroll: ScrollHandle::new(),
            dialog_open: false,
            info: None,
            pending: None,
            status: "Ready".into(),
            error: false,
        }
    }
    fn report<T>(&mut self, result: EditorResult<T>, message: &str, cx: &mut Context<Self>) {
        match result {
            Ok(_) => {
                self.status = message.into();
                self.error = false;
            }
            Err(e) => {
                self.status = e.to_string();
                self.error = true;
            }
        }
        self.request_render(cx);
        cx.notify();
    }
    fn request_render(&mut self, cx: &mut Context<Self>) {
        if self.render_size[0] == 0 {
            return;
        }
        if self.rendering {
            self.render_pending = true;
            return;
        }
        self.rendering = true;
        self.render_pending = false;
        let snapshot = self.editor.engine.snapshot();
        let camera = self.editor.camera;
        let interactive = self.interactive_render;
        let options = RenderOptions {
            width: if interactive {
                self.render_size[0].div_ceil(2)
            } else {
                self.render_size[0]
            },
            height: if interactive {
                self.render_size[1].div_ceil(2)
            } else {
                self.render_size[1]
            },
            supersampling: if interactive { 1 } else { 2 },
            view: View::Camera(self.editor.camera.camera()),
            background: [0; 4],
            ..Default::default()
        };
        let task = cx
            .background_executor()
            .spawn(async move { voxelite_render::render(&snapshot.content, &options) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.rendering = false;
                // Show each completed navigation frame, then coalesce to the newest camera.
                {
                    match result {
                        Ok(mut frame) => {
                            for pixel in frame.rgba.as_chunks_mut::<4>().0 {
                                pixel.swap(0, 2);
                            }
                            let buffer =
                                image::RgbaImage::from_raw(frame.width, frame.height, frame.rgba)
                                    .expect("renderer dimensions");
                            this.display_camera = camera;
                            this.image =
                                Some(Arc::new(RenderImage::new(vec![image::Frame::new(buffer)])));
                        }
                        Err(e) => {
                            this.error = true;
                            this.status = e.to_string();
                        }
                    }
                }
                if this.render_pending {
                    this.request_render(cx);
                }
                cx.notify();
            });
        })
        .detach();
    }
    fn request_navigation(&mut self, cx: &mut Context<Self>) {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        let epoch = self.navigation_epoch;
        self.interactive_render = true;
        self.request_render(cx);
        let timer = cx
            .background_executor()
            .timer(std::time::Duration::from_millis(180));
        cx.spawn(async move |this, cx| {
            timer.await;
            let _ = this.update(cx, |this, cx| {
                if this.navigation_epoch == epoch {
                    this.interactive_render = false;
                    this.request_render(cx);
                }
            });
        })
        .detach();
    }
    fn active_palette(&self) -> Palette {
        PALETTES[self.palette_index]
            .1
            .map(|p| p.palette())
            .unwrap_or_else(|| self.editor.snapshot().palette.clone())
    }
    fn filtered_actions(&self) -> Vec<Op> {
        MENUS
            .iter()
            .flat_map(|(_, items)| items.iter().copied())
            .filter(|op| {
                *op != Op::Commands
                    && op
                        .label()
                        .to_lowercase()
                        .contains(&self.query.to_lowercase())
            })
            .collect()
    }
    fn enabled(&self, op: Op) -> bool {
        match op {
            Op::Undo => self.editor.engine.head().is_some(),
            Op::Redo => self.editor.engine.redo_head().is_some(),
            _ => true,
        }
    }
    fn selected(&self, op: Op) -> bool {
        match op {
            Op::Place => self.editor.tool == Tool::Place,
            Op::Erase => self.editor.tool == Tool::Erase,
            Op::Paint => self.editor.tool == Tool::Paint,
            Op::Fill => self.editor.tool == Tool::Fill,
            Op::Box => self.editor.tool == Tool::Box,
            Op::Picker => self.editor.tool == Tool::Picker,
            Op::Grid => self.grid,
            Op::MirrorX => self.editor.mirror[0],
            Op::MirrorY => self.editor.mirror[1],
            Op::MirrorZ => self.editor.mirror[2],
            _ => false,
        }
    }
    fn invoke(&mut self, op: Op, window: &mut Window, cx: &mut Context<Self>) {
        self.menu = None;
        self.command_open = false;
        self.palette_open = false;
        if self.pending.is_some() || self.dialog_open {
            return;
        }
        if matches!(op, Op::New | Op::Open | Op::Quit) && self.editor.engine.is_dirty() {
            self.pending = Some(op);
            cx.notify();
            return;
        }
        self.run(op, window, cx);
    }
    fn run(&mut self, op: Op, window: &mut Window, cx: &mut Context<Self>) {
        match op {
            Op::Quit => cx.quit(),
            Op::New => match Editor::blank() {
                Ok(editor) => {
                    self.editor = editor;
                    self.image = None;
                    self.hover = None;
                    self.cursor = [0; 3];
                    self.status = "New project".into();
                    self.request_render(cx);
                }
                Err(e) => {
                    self.status = e.to_string();
                    self.error = true;
                }
            },
            Op::Open => self.open_dialog(cx),
            Op::Save => self.save(false, cx),
            Op::SaveAs => self.save(true, cx),
            Op::Undo => {
                let r = self.editor.undo();
                self.report(r, "Undone", cx);
            }
            Op::Redo => {
                let r = self.editor.redo();
                self.report(r, "Redone", cx);
            }
            Op::Theme => self.theme.toggle(),
            Op::Grid => self.grid = !self.grid,
            Op::Frame | Op::Front | Op::Right | Op::Top => {
                let b = self.viewport.get();
                let aspect = (f32::from(b.size.width) / f32::from(b.size.height).max(1.0)) as f64;
                self.editor.camera = OrbitCamera::fit(self.editor.snapshot(), aspect);
                match op {
                    Op::Front => {
                        self.editor.camera.yaw = 0.0;
                        self.editor.camera.pitch = 0.0;
                    }
                    Op::Right => {
                        self.editor.camera.yaw = std::f64::consts::FRAC_PI_2;
                        self.editor.camera.pitch = 0.0;
                    }
                    Op::Top => {
                        self.editor.camera.yaw = 0.0;
                        self.editor.camera.pitch = 1.5;
                    }
                    _ => {}
                }
                self.request_render(cx);
            }
            Op::Place | Op::Erase | Op::Paint | Op::Fill | Op::Box | Op::Picker => {
                self.editor.tool = match op {
                    Op::Erase => Tool::Erase,
                    Op::Paint => Tool::Paint,
                    Op::Fill => Tool::Fill,
                    Op::Box => Tool::Box,
                    Op::Picker => Tool::Picker,
                    _ => Tool::Place,
                };
                self.editor.box_start = None;
                self.status = if op == Op::Box {
                    "Box: click the first corner, then the opposite corner. Esc cancels.".into()
                } else {
                    format!("{} tool", self.editor.tool.name())
                };
                self.error = false;
            }
            Op::MirrorX | Op::MirrorY | Op::MirrorZ => {
                let a = match op {
                    Op::MirrorX => 0,
                    Op::MirrorY => 1,
                    _ => 2,
                };
                self.editor.mirror[a] = !self.editor.mirror[a];
            }
            Op::AddLayer => {
                let r = self.editor.add_layer();
                self.report(r, "Layer added", cx);
            }
            Op::Palette => self.palette_open = true,
            Op::Commands => {
                self.command_open = true;
                self.query.clear();
                self.command_index = 0;
            }
            Op::Agent => {
                self.info = Some((
                    "Agent connection",
                    "This native stage edits a local document. The separate voxelite-mcp server is available for headless agent workflows. Live session attachment is not connected in this shell yet.",
                ))
            }
            Op::Extensions => {
                self.info = Some((
                    "Extensions",
                    "The native shell currently includes six built-in tools. Runtime extension loading belongs to the plugin stage.",
                ))
            }
            Op::Help => {
                self.info = Some((
                    "Make yourself at home",
                    "B Place · E Erase · P Paint · G Fill · U Box · I Picker\nClick to apply · Alt-click to sample a color\nRight-drag to orbit · Middle-drag or Shift-right-drag to pan\nScroll to zoom · F to frame the model\nBox: click two corners · Escape cancels\nArrow keys move X/Z · Page Up/Down move Y · Enter applies\n[ / ] cycle colors · Ctrl/Cmd K searches commands\nCtrl/Cmd Z undoes · Shift Ctrl/Cmd Z redoes\nCtrl/Cmd O opens · Ctrl/Cmd S saves",
                ))
            }
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }
    fn open_dialog(&mut self, cx: &mut Context<Self>) {
        self.dialog_open = true;
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open .voxl project".into()),
        });
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |this, cx| {
                this.dialog_open = false;
                match result {
                    Ok(Ok(Some(paths))) => if let Some(path) = paths.first() { match Editor::open(path) {
                        Ok(editor) => { this.editor = editor; this.cursor = this.editor.snapshot().bounds.min; this.hover = None; this.image = None; this.status = "Project opened".into(); this.error = false; this.request_render(cx); }
                        Err(e) => { this.status = e.to_string(); this.error = true; }
                    } },
                    Ok(Ok(None)) => {},
                    _ => { this.status = "File dialog unavailable. Start voxelite with a .voxl path, or enable your desktop file portal.".into(); this.error = true; }
                } cx.notify();
            });
        }).detach();
    }
    fn save(&mut self, save_as: bool, cx: &mut Context<Self>) {
        if !save_as && let Some(path) = self.editor.path.clone() {
            let result = self.editor.save(&path);
            self.report(result, "Saved", cx);
            return;
        }
        let directory = self
            .editor
            .path
            .as_ref()
            .and_then(|p| p.parent())
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let suggested = format!("{}.voxl", self.editor.snapshot().name);
        self.dialog_open = true;
        let receiver = cx.prompt_for_new_path(&directory, Some(&suggested));
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |this, cx| {
                this.dialog_open = false;
                match result {
                    Ok(Ok(Some(mut path))) => { if path.extension().is_none() { path.set_extension("voxl"); } let r = this.editor.save(&path); this.report(r, "Saved", cx); }
                    Ok(Ok(None)) => {},
                    _ => { this.status = "Save dialog unavailable. Enable a desktop file portal; edits are still open.".into(); this.error = true; }
                } cx.notify();
            });
        }).detach();
    }
    fn ray(&self, p: Point<Pixels>) -> Option<voxelite_app::editor::Ray> {
        // Pointer edits must use the exact camera of the image the user can see.
        if self.rendering {
            return None;
        }
        let b = self.viewport.get();
        let w = f64::from(f32::from(b.size.width));
        let h = f64::from(f32::from(b.size.height));
        self.editor.camera.ray(
            [
                f64::from(f32::from(p.x - b.origin.x)) / w,
                f64::from(f32::from(p.y - b.origin.y)) / h,
            ],
            w / h,
        )
    }
    fn pointer_move(&mut self, e: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.command_open || self.info.is_some() || self.pending.is_some() || self.dialog_open {
            return;
        }
        if matches!(
            e.pressed_button,
            Some(MouseButton::Right | MouseButton::Middle)
        ) {
            if let Some(previous) = self.drag {
                let dx = f64::from(f32::from(e.position.x - previous.x));
                let dy = f64::from(f32::from(e.position.y - previous.y));
                if e.pressed_button == Some(MouseButton::Middle) || e.modifiers.shift {
                    self.editor.camera.pan(
                        dx,
                        dy,
                        f64::from(f32::from(self.viewport.get().size.height)),
                    );
                } else {
                    self.editor.camera.orbit(dx, dy);
                }
                self.request_navigation(cx);
            }
            self.drag = Some(e.position);
            self.hover = None;
        } else {
            self.drag = None;
            self.hover = self.ray(e.position).and_then(|r| self.editor.target(r));
        }
        cx.notify();
    }
    fn key_down(&mut self, e: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = e.keystroke.key.as_str();
        if key == "escape" {
            self.menu = None;
            self.command_open = false;
            self.palette_open = false;
            self.info = None;
            self.pending = None;
            self.editor.box_start = None;
            self.status = "Ready".into();
            self.error = false;
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.command_open {
            let count = self.filtered_actions().len();
            match key {
                "backspace" => {
                    self.query.pop();
                    self.command_index = 0;
                }
                "down" => {
                    self.command_index = (self.command_index + 1).min(count.saturating_sub(1))
                }
                "up" => self.command_index = self.command_index.saturating_sub(1),
                "enter" => {
                    if let Some(op) = self.filtered_actions().get(self.command_index).copied() {
                        self.invoke(op, window, cx);
                    }
                }
                _ => {
                    if let Some(text) = &e.keystroke.key_char {
                        self.query.push_str(text);
                        self.command_index = 0;
                    }
                }
            }
            self.command_scroll.scroll_to_item(self.command_index);
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if let Some(menu) = self.menu {
            let items = MENUS[menu].1;
            match key {
                "down" => self.menu_index = (self.menu_index + 1) % items.len(),
                "up" => self.menu_index = (self.menu_index + items.len() - 1) % items.len(),
                "right" => {
                    self.menu = Some((menu + 1) % MENUS.len());
                    self.menu_index = 0;
                }
                "left" => {
                    self.menu = Some((menu + MENUS.len() - 1) % MENUS.len());
                    self.menu_index = 0;
                }
                "enter" => self.invoke(items[self.menu_index], window, cx),
                _ => {}
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.info.is_some() || self.pending.is_some() || self.palette_open {
            cx.stop_propagation();
            return;
        }
        let movement = match key {
            "left" => Some((0, -1)),
            "right" => Some((0, 1)),
            "up" => Some((2, -1)),
            "down" => Some((2, 1)),
            "pageup" => Some((1, 1)),
            "pagedown" => Some((1, -1)),
            _ => None,
        };
        if let Some((a, delta)) = movement {
            self.cursor[a] = (self.cursor[a] + delta).clamp(
                self.editor.snapshot().bounds.min[a],
                self.editor.snapshot().bounds.max[a] - 1,
            );
            self.hover = Some(self.cursor);
            cx.stop_propagation();
        }
        if key == "enter" {
            let r = self.editor.apply_at(self.cursor);
            self.report(r, "Tool applied", cx);
            cx.stop_propagation();
        }
        if key == "[" || key == "]" {
            let palette = self.active_palette();
            let index = palette
                .order
                .iter()
                .position(|slot| {
                    palette
                        .get(*slot)
                        .is_some_and(|s| s.rgba == self.editor.color.rgba)
                })
                .unwrap_or(0);
            let index = if key == "]" {
                (index + 1) % palette.order.len()
            } else {
                (index + palette.order.len() - 1) % palette.order.len()
            };
            self.editor.color = palette
                .get(palette.order[index])
                .expect("valid palette")
                .clone();
            cx.stop_propagation();
        }
        cx.notify();
    }
    fn button(
        &self,
        id: &'static str,
        name: &'static str,
        label: &'static str,
        op: Op,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let theme = self.theme;
        let enabled = self.enabled(op);
        row()
            .id(id)
            .tooltip(move |_, cx| {
                cx.new(|_| Hint {
                    text: format!("{}  {}", op.label(), op.shortcut()).into(),
                    theme,
                })
                .into()
            })
            .h(px(32.0))
            .px(px(9.0))
            .gap(px(8.0))
            .rounded(px(6.0))
            .cursor_pointer()
            .text_color(if enabled { theme.muted() } else { theme.line() })
            .when(self.selected(op), |d| {
                d.bg(theme.selected()).text_color(theme.accent())
            })
            .hover(move |d| d.bg(theme.hover()).text_color(theme.text()))
            .child(icon(name).text_color(if enabled {
                theme.muted()
            } else {
                theme.muted().opacity(0.4)
            }))
            .when(!label.is_empty(), |d| d.child(label))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, window, cx| {
                if enabled {
                    this.invoke(op, window, cx);
                }
            }))
    }
    fn menubar(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .h(px(34.0))
            .flex_shrink_0()
            .px(px(18.0))
            .border_b_1()
            .border_color(t.line())
            .gap(px(3.0))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .mr(px(18.0))
                    .child("Voxelite"),
            )
            .children(MENUS.iter().enumerate().map(|(i, (name, _))| {
                row()
                    .id(("menu", i))
                    .h(px(28.0))
                    .px(px(10.0))
                    .rounded(px(5.0))
                    .cursor_pointer()
                    .child(*name)
                    .when(self.menu == Some(i), |d| d.bg(t.hover()))
                    .hover(move |d| d.bg(t.hover()))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.menu = if this.menu == Some(i) { None } else { Some(i) };
                        this.menu_index = 0;
                        this.command_open = false;
                        cx.notify();
                    }))
            }))
            .child(div().flex_1())
            .child(mono("NATIVE / v0.1").text_color(t.muted()))
    }
    fn toolbar(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        row()
            .h(px(62.0))
            .flex_shrink_0()
            .px(px(20.0))
            .gap(px(12.0))
            .border_b_1()
            .border_color(t.line())
            .child(icon("cube").text_color(t.muted()).size(px(22.0)))
            .child(
                div()
                    .min_w(px(180.0))
                    .child(
                        row()
                            .gap(px(5.0))
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.editor.snapshot().name.clone())
                            .child(div().text_color(t.muted()).child(".voxl")),
                    )
                    .child(div().text_size(px(12.0)).text_color(t.muted()).child(
                        if self.editor.engine.is_dirty() {
                            "Unsaved changes"
                        } else if self.editor.path.is_some() {
                            "Saved locally"
                        } else {
                            "Example project"
                        },
                    )),
            )
            .child(div().w(px(1.0)).h(px(26.0)).bg(t.line()).mx(px(8.0)))
            .child(self.button("undo", "undo", "", Op::Undo, cx))
            .child(self.button("redo", "redo", "", Op::Redo, cx))
            .child(self.button("frame", "fit", "Frame", Op::Frame, cx))
            .child(div().flex_1())
            .child(
                self.button("commands", "search", "Search commands", Op::Commands, cx)
                    .child(
                        mono(if cfg!(target_os = "macos") {
                            "⌘ K"
                        } else {
                            "Ctrl K"
                        })
                        .ml(px(20.0)),
                    ),
            )
            .child(div().w(px(1.0)).h(px(26.0)).bg(t.line()).mx(px(4.0)))
            .child(self.button(
                "theme",
                if t == Theme::Dark { "sun" } else { "moon" },
                "",
                Op::Theme,
                cx,
            ))
            .child(
                self.button("save", "save", "Save", Op::Save, cx)
                    .border_1()
                    .border_color(t.line())
                    .text_color(t.text()),
            )
    }
    fn tool_rail(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let ops = [
            Op::Place,
            Op::Erase,
            Op::Paint,
            Op::Fill,
            Op::Box,
            Op::Picker,
        ];
        let icons = ["place", "erase", "paint", "fill", "box", "picker"];
        div()
            .w(px(72.0))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .py(px(16.0))
            .gap(px(6.0))
            .border_r_1()
            .border_color(t.line())
            .children(Tool::ALL.into_iter().enumerate().map(|(i, tool)| {
                let op = ops[i];
                div()
                    .id(("tool", i))
                    .tooltip(move |_, cx| {
                        cx.new(|_| Hint {
                            text: format!("{} · {}", tool.name(), tool.key()).into(),
                            theme: t,
                        })
                        .into()
                    })
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .w(px(56.0))
                    .h(px(58.0))
                    .rounded(px(7.0))
                    .gap(px(5.0))
                    .cursor_pointer()
                    .text_color(if self.editor.tool == tool {
                        t.accent()
                    } else {
                        t.muted()
                    })
                    .when(self.editor.tool == tool, |d| {
                        d.bg(t.selected())
                            .border_1()
                            .border_color(t.accent().opacity(0.35))
                    })
                    .hover(move |d| d.bg(t.hover()))
                    .child(
                        icon(icons[i])
                            .size(px(20.0))
                            .text_color(if self.editor.tool == tool {
                                t.accent()
                            } else {
                                t.muted()
                            }),
                    )
                    .child(div().text_size(px(12.0)).child(tool.name()))
                    .on_click(cx.listener(move |this, _, window, cx| this.invoke(op, window, cx)))
            }))
            .child(div().h(px(1.0)).w(px(32.0)).my(px(9.0)).bg(t.line()))
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(t.muted())
                    .child("Mirror"),
            )
            .children(
                [Op::MirrorX, Op::MirrorY, Op::MirrorZ]
                    .into_iter()
                    .enumerate()
                    .map(|(i, op)| {
                        row()
                            .id(("mirror", i))
                            .w(px(36.0))
                            .h(px(32.0))
                            .justify_center()
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .font_family("Geist Mono")
                            .text_color(if self.editor.mirror[i] {
                                t.accent()
                            } else {
                                t.muted()
                            })
                            .when(self.editor.mirror[i], |d| {
                                d.bg(t.selected()).border_1().border_color(t.accent())
                            })
                            .hover(move |d| d.bg(t.hover()))
                            .child(["X", "Y", "Z"][i])
                            .on_click(
                                cx.listener(move |this, _, window, cx| this.invoke(op, window, cx)),
                            )
                    }),
            )
            .child(div().flex_1())
            .child(self.button("help", "spark", "", Op::Help, cx))
    }
    fn viewport(&self, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let bounds_cell = self.viewport.clone();
        let image = self.image.clone();
        let camera = self.display_camera;
        let hover = self.hover;
        let grid = self.grid;
        let doc_bounds = self.editor.snapshot().bounds;
        let box_start = self.editor.box_start;
        let surface = canvas(
            move |bounds, _, _| {
                bounds_cell.set(bounds);
            },
            move |bounds, _, window, _| {
                let aspect =
                    f64::from(f32::from(bounds.size.width) / f32::from(bounds.size.height));
                let project = |p: DVec3| {
                    let uv = camera.project(p, aspect);
                    point(
                        bounds.origin.x + bounds.size.width * uv[0] as f32,
                        bounds.origin.y + bounds.size.height * uv[1] as f32,
                    )
                };
                if grid {
                    let mut path = PathBuilder::stroke(px(0.6));
                    let y = f64::from(doc_bounds.min[1]) - 0.05;
                    let extent = doc_bounds.min[0].min(doc_bounds.min[2]) - 16;
                    let end = doc_bounds.max[0].max(doc_bounds.max[2]) + 16;
                    for a in (extent..=end).step_by(4) {
                        path.move_to(project(DVec3::new(f64::from(a), y, f64::from(extent))));
                        path.line_to(project(DVec3::new(f64::from(a), y, f64::from(end))));
                        path.move_to(project(DVec3::new(f64::from(extent), y, f64::from(a))));
                        path.line_to(project(DVec3::new(f64::from(end), y, f64::from(a))));
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, t.line().opacity(0.35));
                    }
                }
                if let Some(image) = image {
                    let _ = window.paint_image(bounds, bounds, Corners::default(), image, 0, false);
                }
                if let Some(at) = hover.or(box_start) {
                    let lo = box_start
                        .map(|s| std::array::from_fn(|a| s[a].min(at[a])))
                        .unwrap_or(at);
                    let hi: Pos = box_start
                        .map(|s| std::array::from_fn(|a| s[a].max(at[a]) + 1))
                        .unwrap_or(at.map(|v| v + 1));
                    let mut path = PathBuilder::stroke(px(1.4));
                    for bits in 0..8 {
                        for a in 0..3 {
                            if bits & (1 << a) == 0 {
                                let p: [f64; 3] = std::array::from_fn(|j| {
                                    f64::from(if bits & (1 << j) == 0 { lo[j] } else { hi[j] })
                                });
                                let mut q = p;
                                q[a] = f64::from(hi[a]);
                                path.move_to(project(p.into()));
                                path.line_to(project(q.into()));
                            }
                        }
                    }
                    if let Ok(path) = path.build() {
                        window.paint_path(path, t.accent());
                    }
                }
            },
        )
        .size_full();
        div()
            .id("viewport")
            .relative()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .cursor_crosshair()
            .child(surface)
            .child(
                div().absolute().top(px(20.0)).left(px(24.0)).child(
                    row()
                        .gap(px(10.0))
                        .text_color(t.muted())
                        .child("Orthographic")
                        .child(div().w(px(3.0)).h(px(3.0)).rounded_full().bg(t.muted()))
                        .child("Studio lighting"),
                ),
            )
            .child(
                div()
                    .absolute()
                    .top(px(18.0))
                    .right(px(20.0))
                    .child(self.button("view-fit", "fit", "", Op::Frame, cx)),
            )
            .child(
                div()
                    .absolute()
                    .bottom(px(20.0))
                    .left(px(24.0))
                    .text_color(t.muted())
                    .text_size(px(12.0))
                    .child("Right-drag orbit   ·   Shift-drag pan   ·   Scroll zoom"),
            )
            .child(
                div().absolute().bottom(px(14.0)).right(px(20.0)).child(
                    row()
                        .gap(px(4.0))
                        .p(px(4.0))
                        .bg(t.background())
                        .border_1()
                        .border_color(t.line())
                        .rounded(px(7.0))
                        .child(self.button("view-front", "cube", "Front", Op::Front, cx))
                        .child(self.button("view-right", "cube", "Right", Op::Right, cx))
                        .child(self.button("view-top", "cube", "Top", Op::Top, cx)),
                ),
            )
            .on_mouse_move(cx.listener(Self::pointer_move))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                    this.drag = Some(e.position);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                    this.drag = Some(e.position);
                    cx.stop_propagation();
                }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    window.focus(&this.focus, cx);
                    if this.menu.take().is_some() || this.palette_open {
                        this.palette_open = false;
                        cx.notify();
                        return;
                    }
                    if this.command_open
                        || this.info.is_some()
                        || this.pending.is_some()
                        || this.dialog_open
                    {
                        return;
                    }
                    if let Some(ray) = this.ray(e.position) {
                        let r = this.editor.click(ray, e.modifiers.alt);
                        this.report(
                            r,
                            if this.editor.box_start.is_some() {
                                "First corner set · click opposite corner · Esc cancels"
                            } else {
                                "Ready"
                            },
                            cx,
                        );
                    }
                }),
            )
            .on_scroll_wheel(cx.listener(|this, e: &ScrollWheelEvent, _, cx| {
                let delta = match e.delta {
                    ScrollDelta::Pixels(p) => f64::from(f32::from(p.y)),
                    ScrollDelta::Lines(p) => f64::from(p.y) * 40.0,
                };
                this.editor.camera.zoom(delta);
                this.request_navigation(cx);
                cx.notify();
                cx.stop_propagation();
            }))
    }
    fn palette_panel(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        let palette = self.active_palette();
        let color = self.editor.color.rgba;
        let hex = format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
        div()
            .flex_shrink_0()
            .p(px(20.0))
            .border_b_1()
            .border_color(t.line())
            .child(
                row()
                    .justify_between()
                    .mb(px(14.0))
                    .child(div().font_weight(FontWeight::MEDIUM).child("Palette"))
                    .child(mono(format!("{} colors", palette.order.len())).text_color(t.muted())),
            )
            .child(
                row()
                    .id("palette-switch")
                    .h(px(36.0))
                    .px(px(10.0))
                    .mb(px(14.0))
                    .border_1()
                    .border_color(t.line())
                    .rounded(px(6.0))
                    .justify_between()
                    .cursor_pointer()
                    .hover(move |d| d.bg(t.hover()))
                    .child(PALETTES[self.palette_index].0)
                    .child(icon("chevron").text_color(t.muted()))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.palette_open = !this.palette_open;
                        cx.notify();
                    })),
            )
            .child(div().flex().flex_wrap().gap(px(6.0)).children(
                palette.order.iter().enumerate().map(|(i, slot)| {
                    let swatch = palette.get(*slot).expect("valid palette").clone();
                    let c = swatch.rgba;
                    let selected = c == color;
                    div()
                        .id(("swatch", i))
                        .w(px(26.0))
                        .h(px(26.0))
                        .p(px(2.0))
                        .rounded(px(6.0))
                        .border_1()
                        .border_color(if selected { t.accent() } else { t.line() })
                        .cursor_pointer()
                        .hover(move |d| d.border_color(t.text()))
                        .child(
                            div()
                                .size_full()
                                .rounded(px(3.0))
                                .bg(rgb(u32::from(c[0]) << 16
                                    | u32::from(c[1]) << 8
                                    | u32::from(c[2]))),
                        )
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.editor.color = swatch.clone();
                            this.error = false;
                            cx.notify();
                        }))
                }),
            ))
            .child(
                row()
                    .mt(px(14.0))
                    .gap(px(8.0))
                    .text_color(t.muted())
                    .child(
                        div()
                            .size(px(12.0))
                            .rounded(px(3.0))
                            .border_1()
                            .border_color(t.line())
                            .bg(rgb(u32::from(color[0]) << 16
                                | u32::from(color[1]) << 8
                                | u32::from(color[2]))),
                    )
                    .child(mono(hex))
                    .child(div().flex_1())
                    .child(mono("[  ]")),
            )
    }
    fn layers_panel(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        div()
            .flex_shrink_0()
            .px(px(16.0))
            .py(px(16.0))
            .border_b_1()
            .border_color(t.line())
            .child(
                row()
                    .px(px(4.0))
                    .mb(px(8.0))
                    .justify_between()
                    .child(div().font_weight(FontWeight::MEDIUM).child("Layers"))
                    .child(self.button("add-layer", "plus", "", Op::AddLayer, cx)),
            )
            .child(
                div()
                    .id("layer-scroll")
                    .max_h(px(240.0))
                    .overflow_y_scroll()
                    .children(self.editor.snapshot().layers.iter().rev().enumerate().map(
                        |(i, layer)| {
                            let id = layer.id;
                            let visible = layer.visible;
                            let locked = layer.locked;
                            row()
                                .id(("layer", i))
                                .h(px(38.0))
                                .px(px(8.0))
                                .gap(px(9.0))
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .when(id == self.editor.layer, |d| d.bg(t.hover()))
                                .text_color(if visible { t.text() } else { t.muted() })
                                .child(icon("layer").size(px(16.0)).text_color(t.muted()))
                                .child(
                                    div()
                                        .flex_1()
                                        .overflow_hidden()
                                        .text_ellipsis()
                                        .child(layer.name.clone()),
                                )
                                .child(
                                    row()
                                        .id(("eye", i))
                                        .w(px(28.0))
                                        .h(px(32.0))
                                        .justify_center()
                                        .rounded(px(5.0))
                                        .hover(move |d| d.bg(t.selected()))
                                        .child(
                                            icon(if visible { "eye" } else { "eye-off" })
                                                .size(px(15.0))
                                                .text_color(t.muted()),
                                        )
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            cx.stop_propagation();
                                            let r =
                                                this.editor.update_layer(id, Some(!visible), None);
                                            this.report(r, "Visibility updated", cx);
                                        })),
                                )
                                .child(
                                    row()
                                        .id(("lock", i))
                                        .w(px(28.0))
                                        .h(px(32.0))
                                        .justify_center()
                                        .rounded(px(5.0))
                                        .hover(move |d| d.bg(t.selected()))
                                        .child(
                                            icon(if locked { "lock" } else { "unlock" })
                                                .size(px(14.0))
                                                .text_color(if locked {
                                                    t.accent()
                                                } else {
                                                    t.muted()
                                                }),
                                        )
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            cx.stop_propagation();
                                            let r =
                                                this.editor.update_layer(id, None, Some(!locked));
                                            this.report(r, "Lock updated", cx);
                                        })),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.editor.layer = id;
                                    this.editor.box_start = None;
                                    cx.notify();
                                }))
                        },
                    )),
            )
    }
    fn history_panel(&self, cx: &Context<Self>) -> Div {
        let t = self.theme;
        div()
            .flex_1()
            .min_h(px(100.0))
            .flex()
            .flex_col()
            .p(px(20.0))
            .gap(px(14.0))
            .child(
                row()
                    .justify_between()
                    .child(div().font_weight(FontWeight::MEDIUM).child("History"))
                    .child(
                        mono(format!("r{:03}", self.editor.engine.revision()))
                            .text_color(t.muted()),
                    ),
            )
            .child(
                div()
                    .id("history-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .children(
                        self.editor
                            .engine
                            .history()
                            .rev()
                            .take(100)
                            .enumerate()
                            .map(|(i, entry)| {
                                row()
                                    .id(("history", i))
                                    .py(px(8.0))
                                    .gap(px(10.0))
                                    .child(icon("check").size(px(14.0)).text_color(t.muted()))
                                    .child(
                                        div().flex_1().child(entry.label.clone()).child(
                                            mono(format!(
                                                "Human · +{} −{} · {} painted",
                                                entry.changes.added,
                                                entry.changes.removed,
                                                entry.changes.recolored
                                            ))
                                            .text_color(t.muted())
                                            .mt(px(4.0)),
                                        ),
                                    )
                            }),
                    )
                    .child(
                        row()
                            .py(px(8.0))
                            .gap(px(10.0))
                            .child(icon("history").size(px(14.0)).text_color(t.muted()))
                            .child(
                                div().child("Project opened").child(
                                    div()
                                        .text_size(px(12.0))
                                        .mt(px(4.0))
                                        .text_color(t.muted())
                                        .child("Your edits will appear here"),
                                ),
                            ),
                    ),
            )
            .child(
                self.button("history-undo", "undo", "Undo last edit", Op::Undo, cx)
                    .w_full()
                    .justify_center()
                    .border_1()
                    .border_color(t.line()),
            )
    }
    fn sidebar(&self, cx: &Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        div()
            .id("sidebar-scroll")
            .overflow_y_scroll()
            .w(px(296.0))
            .flex_shrink_0()
            .h_full()
            .border_l_1()
            .border_color(t.line())
            .flex()
            .flex_col()
            .child(self.palette_panel(cx))
            .child(self.layers_panel(cx))
            .child(self.history_panel(cx))
            .child(
                row()
                    .h(px(48.0))
                    .flex_shrink_0()
                    .px(px(20.0))
                    .gap(px(9.0))
                    .border_t_1()
                    .border_color(t.line())
                    .text_color(t.muted())
                    .child(icon("spark"))
                    .child("Local session")
                    .child(div().flex_1())
                    .child(mono("Human")),
            )
    }
    fn statusbar(&self) -> Div {
        let t = self.theme;
        let s = self.editor.snapshot();
        let size: [i32; 3] = std::array::from_fn(|a| s.bounds.max[a] - s.bounds.min[a]);
        row()
            .h(px(30.0))
            .flex_shrink_0()
            .border_t_1()
            .border_color(t.line())
            .px(px(18.0))
            .gap(px(16.0))
            .text_size(px(12.0))
            .text_color(t.muted())
            .child(mono(format!(
                "{} voxels",
                s.layers.iter().map(|l| l.voxels.len()).sum::<usize>()
            )))
            .child(mono(format!("{} × {} × {}", size[0], size[1], size[2])))
            .child(div().h(px(12.0)).w(px(1.0)).bg(t.line()))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_color(if self.error { t.danger() } else { t.muted() })
                    .child(self.status.clone()),
            )
            .when_some(self.hover, |d, p| {
                d.child(mono(format!("X {}   Y {}   Z {}", p[0], p[1], p[2])))
            })
            .child(mono(if self.rendering {
                "Rendering…"
            } else {
                "CPU · 2× AA"
            }))
    }
    fn action_row(
        &self,
        op: Op,
        index: usize,
        selected: bool,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let t = self.theme;
        let enabled = self.enabled(op);
        row()
            .id(("action-row", index))
            .h(px(36.0))
            .px(px(10.0))
            .gap(px(10.0))
            .rounded(px(5.0))
            .cursor_pointer()
            .when(selected, |d| d.bg(t.hover()))
            .text_color(if enabled {
                t.text()
            } else {
                t.muted().opacity(0.5)
            })
            .hover(move |d| d.bg(t.hover()))
            .child(div().w(px(14.0)).when(self.selected(op), |d| {
                d.child(icon("check").size(px(14.0)).text_color(t.accent()))
            }))
            .child(op.label())
            .child(div().flex_1())
            .child(mono(op.shortcut()).text_color(t.muted()))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(cx.listener(move |this, _, window, cx| {
                if enabled {
                    this.invoke(op, window, cx);
                }
            }))
    }
    fn floating(&self) -> Div {
        div()
            .bg(self.theme.background())
            .border_1()
            .border_color(self.theme.line())
            .rounded(px(8.0))
            .shadow_lg()
            .p(px(6.0))
    }
    fn overlay(&self, cx: &Context<Self>) -> Option<AnyElement> {
        let t = self.theme;
        if let Some(op) = self.pending {
            return Some(div().absolute().size_full().flex().justify_center().items_center().bg(rgba(0x00000070))
                .child(self.floating().w(px(440.0)).p(px(24.0)).child(div().text_size(px(18.0)).font_weight(FontWeight::SEMIBOLD).child("Save your changes?"))
                    .child(div().my(px(16.0)).text_color(t.muted()).child("This project has unsaved edits. Save them or explicitly discard them before continuing."))
                    .child(row().justify_end().gap(px(8.0))
                        .child(row().id("discard").h(px(36.0)).px(px(10.0)).rounded(px(6.0)).text_color(t.danger()).cursor_pointer().hover(move |d| d.bg(t.hover())).child("Discard").on_click(cx.listener(move |this, _, window, cx| { this.pending = None; this.run(op, window, cx); })))
                        .child(row().id("cancel").h(px(36.0)).px(px(10.0)).rounded(px(6.0)).cursor_pointer().hover(move |d| d.bg(t.hover())).child("Cancel").on_click(cx.listener(|this, _, _, cx| { this.pending = None; cx.notify(); })))
                        .child(row().id("save-keep").h(px(36.0)).px(px(12.0)).rounded(px(6.0)).border_1().border_color(t.line()).cursor_pointer().hover(move |d| d.bg(t.hover())).child("Save & keep editing").on_click(cx.listener(|this, _, _, cx| { this.pending = None; this.save(false, cx); })))
                    )).into_any_element());
        }
        if let Some((title, text)) = self.info {
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .justify_center()
                    .items_center()
                    .bg(rgba(0x00000070))
                    .child(
                        self.floating()
                            .w(px(560.0))
                            .p(px(24.0))
                            .child(
                                row()
                                    .justify_between()
                                    .mb(px(18.0))
                                    .child(
                                        div()
                                            .text_size(px(18.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(title),
                                    )
                                    .child(
                                        row()
                                            .id("close-info")
                                            .size(px(32.0))
                                            .justify_center()
                                            .cursor_pointer()
                                            .child(icon("close").text_color(t.muted()))
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.info = None;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .text_color(t.muted())
                                    .line_height(px(26.0))
                                    .child(text),
                            ),
                    )
                    .into_any_element(),
            );
        }
        if self.command_open {
            let matches = self.filtered_actions();
            return Some(
                div()
                    .absolute()
                    .size_full()
                    .bg(rgba(0x00000070))
                    .flex()
                    .justify_center()
                    .pt(px(140.0))
                    .child(
                        self.floating()
                            .w(px(560.0))
                            .h(px(440.0))
                            .flex()
                            .flex_col()
                            .child(
                                row()
                                    .h(px(52.0))
                                    .px(px(14.0))
                                    .gap(px(12.0))
                                    .border_b_1()
                                    .border_color(t.line())
                                    .child(icon("search").text_color(t.muted()))
                                    .child(div().flex_1().text_size(px(16.0)).child(
                                        if self.query.is_empty() {
                                            "Type a command…".to_string()
                                        } else {
                                            format!("{}│", self.query)
                                        },
                                    ))
                                    .child(mono("ESC").text_color(t.muted())),
                            )
                            .child(
                                div()
                                    .id("command-results")
                                    .track_scroll(&self.command_scroll)
                                    .flex_1()
                                    .overflow_y_scroll()
                                    .py(px(6.0))
                                    .children(matches.iter().enumerate().map(|(i, op)| {
                                        self.action_row(*op, i, self.command_index == i, cx)
                                    }))
                                    .when(matches.is_empty(), |d| {
                                        d.child(
                                            div()
                                                .p(px(24.0))
                                                .text_color(t.muted())
                                                .child("No matching commands"),
                                        )
                                    }),
                            )
                            .child(
                                row()
                                    .h(px(32.0))
                                    .px(px(14.0))
                                    .text_color(t.muted())
                                    .text_size(px(12.0))
                                    .border_t_1()
                                    .border_color(t.line())
                                    .child("↑ ↓ to navigate   ·   Enter to run   ·   Esc to close"),
                            ),
                    )
                    .into_any_element(),
            );
        }
        if self.palette_open {
            return Some(
                div()
                    .absolute()
                    .top(px(112.0))
                    .right(px(18.0))
                    .child(
                        self.floating()
                            .w(px(280.0))
                            .child(
                                div()
                                    .px(px(10.0))
                                    .py(px(8.0))
                                    .text_color(t.muted())
                                    .text_size(px(12.0))
                                    .child("CHOOSE PALETTE"),
                            )
                            .children(PALETTES.iter().enumerate().map(|(i, (name, preset))| {
                                let p = preset
                                    .map(|p| p.palette())
                                    .unwrap_or_else(|| self.editor.snapshot().palette.clone());
                                div()
                                    .id(("preset", i))
                                    .p(px(10.0))
                                    .rounded(px(6.0))
                                    .cursor_pointer()
                                    .hover(move |d| d.bg(t.hover()))
                                    .child(row().justify_between().mb(px(9.0)).child(*name).when(
                                        i == self.palette_index,
                                        |d| {
                                            d.child(
                                                icon("check").size(px(14.0)).text_color(t.accent()),
                                            )
                                        },
                                    ))
                                    .child(
                                        row()
                                            .h(px(9.0))
                                            .rounded(px(3.0))
                                            .overflow_hidden()
                                            .children(p.order.iter().take(32).map(|slot| {
                                                let c = p.get(*slot).expect("valid palette").rgba;
                                                div()
                                                    .flex_1()
                                                    .h_full()
                                                    .bg(rgb(u32::from(c[0]) << 16
                                                        | u32::from(c[1]) << 8
                                                        | u32::from(c[2])))
                                            })),
                                    )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.palette_index = i;
                                        this.palette_open = false;
                                        cx.notify();
                                    }))
                            })),
                    )
                    .into_any_element(),
            );
        }
        self.menu.map(|i| {
            let left = 100.0
                + MENUS
                    .iter()
                    .take(i)
                    .map(|(n, _)| n.len() as f32 * 7.0 + 23.0)
                    .sum::<f32>();
            div()
                .absolute()
                .top(px(33.0))
                .left(px(left))
                .child(
                    self.floating().w(px(310.0)).children(
                        MENUS[i]
                            .1
                            .iter()
                            .enumerate()
                            .map(|(j, op)| self.action_row(*op, j, j == self.menu_index, cx)),
                    ),
                )
                .into_any_element()
        })
    }
}
impl Render for EditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = window.viewport_size();
        let width = (f32::from(size.width) - 368.0).max(1.0);
        let height = (f32::from(size.height) - 126.0).max(1.0);
        let scale = (1024.0 / width.max(height)).min(1.0);
        let dimensions = [
            (width * scale).round() as u32,
            (height * scale).round() as u32,
        ];
        if self.render_size != dimensions {
            self.render_size = dimensions;
            self.request_render(cx);
        }
        window.set_window_title(&format!(
            "{}{} — Voxelite",
            self.editor.snapshot().name,
            if self.editor.engine.is_dirty() {
                " •"
            } else {
                ""
            }
        ));
        cx.set_menus(
            MENUS
                .iter()
                .map(|(name, items)| {
                    Menu::new(*name).items(items.iter().map(|op| {
                        MenuItem::action(op.label(), *op)
                            .checked(self.selected(*op))
                            .disabled(!self.enabled(*op))
                    }))
                })
                .collect::<Vec<_>>(),
        );
        let overlay = self.overlay(cx);
        div()
            .id("editor-root")
            .key_context(if self.command_open {
                "CommandPalette"
            } else if self.menu.is_some()
                || self.palette_open
                || self.pending.is_some()
                || self.info.is_some()
                || self.dialog_open
            {
                "Modal"
            } else {
                "Editor"
            })
            .track_focus(&self.focus)
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(self.theme.background())
            .text_color(self.theme.text())
            .font_family("Geist")
            .text_size(px(13.0))
            .capture_key_down(cx.listener(Self::key_down))
            .on_action(cx.listener(|this, op: &Op, window, cx| {
                if this.info.is_none() && this.pending.is_none() {
                    this.invoke(*op, window, cx);
                }
            }))
            .child(self.menubar(cx))
            .child(self.toolbar(cx))
            .child(
                row()
                    .flex_1()
                    .min_h(px(0.0))
                    .items_stretch()
                    .child(self.tool_rail(cx))
                    .child(self.viewport(cx))
                    .child(self.sidebar(cx)),
            )
            .child(self.statusbar())
            .children(overlay.map(|overlay| {
                div()
                    .id("overlay")
                    .absolute()
                    .size_full()
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(overlay)
            }))
    }
}
