//! Palette dialog controller. All color and library changes are host-admitted actions.
#[cfg(target_os = "macos")]
mod eyedropper;
mod files;
mod view;
use super::*;
use crate::text_field::TextField;
use voxelite_plugin_api::{PaletteId, PaletteOperation, PaletteOutcome, PaletteView};
use voxelite_ui::{ColorChange, ColorGesture, Hsb, parse_rgb_hex, rgb_hex};

pub(super) struct PaletteManager {
    pub open: bool,
    pub name: Entity<TextField>,
    pub hex: Entity<TextField>,
    channels: [Entity<TextField>; 6],
    synced: [String; 8],
    hsb: Hsb,
    before: [u8; 3],
    preview_hsb: Option<Hsb>,
    identity: Option<(PaletteId, u16)>,
    copied_from: Option<(PaletteId, String)>,
    error: Option<String>,
    menu: bool,
    menu_index: usize,
    delete_confirmation: bool,
    recent_add: bool,
    sidebar_focus: FocusHandle,
    sidebar_scroll: ScrollHandle,
    grid_focus: FocusHandle,
    grid_scroll: ScrollHandle,
    drop_boundary: Option<usize>,
    square_gesture: ColorGesture,
    hue_gesture: ColorGesture,
    #[cfg(target_os = "macos")]
    sampler_failed: bool,
}
impl PaletteManager {
    pub fn new(window: &mut Window, cx: &mut Context<EditorView>) -> Self {
        let hex = TextField::new("#RRGGBB", cx);
        hex.update(cx, |field, _| field.numeric(7));
        let manager = Self {
            open: false,
            name: TextField::new("Palette name", cx),
            hex,
            channels: ["Red", "Green", "Blue", "Hue", "Saturation", "Brightness"].map(|label| {
                let field = TextField::new(label, cx);
                field.update(cx, |field, _| field.numeric(3));
                field
            }),
            synced: Default::default(),
            hsb: Hsb::default(),
            before: [0; 3],
            preview_hsb: None,
            identity: None,
            copied_from: None,
            error: None,
            menu: false,
            menu_index: 0,
            delete_confirmation: false,
            recent_add: false,
            sidebar_focus: cx.focus_handle(),
            sidebar_scroll: ScrollHandle::new(),
            grid_focus: cx.focus_handle(),
            grid_scroll: ScrollHandle::new(),
            drop_boundary: None,
            square_gesture: ColorGesture::default(),
            hue_gesture: ColorGesture::default(),
            #[cfg(target_os = "macos")]
            sampler_failed: false,
        };
        for (index, field) in manager.fields().into_iter().enumerate() {
            cx.on_blur(&field.focus_handle(cx), window, move |this, _, cx| {
                if this.palettes.open {
                    this.commit_palette_field(index, cx);
                }
            })
            .detach();
            // Enter is a text-field action, not a key event the dialog sees.
            cx.subscribe(
                &field,
                move |this, _, event: &voxelite_ui::TextFieldEvent, cx| {
                    if matches!(event, voxelite_ui::TextFieldEvent::Committed) && this.palettes.open
                    {
                        this.commit_palette_field(index, cx);
                        cx.notify();
                    }
                },
            )
            .detach();
        }
        manager
    }
    fn fields(&self) -> [Entity<TextField>; 8] {
        [
            self.name.clone(),
            self.hex.clone(),
            self.channels[0].clone(),
            self.channels[1].clone(),
            self.channels[2].clone(),
            self.channels[3].clone(),
            self.channels[4].clone(),
            self.channels[5].clone(),
        ]
    }
    pub(super) fn field_focused(&self, window: &Window, cx: &App) -> bool {
        self.fields().iter().any(|f| f.read(cx).focused(window))
    }
    pub(super) fn appearance(&self, theme: Theme, cx: &mut App) {
        for field in self.fields() {
            field.update(cx, |f, cx| f.appearance(theme, cx));
        }
    }
}
fn selected_rgb(view: &PaletteView) -> [u8; 3] {
    view.palette
        .order
        .get(view.state.slot)
        .and_then(|slot| view.palette.get(*slot))
        .map_or([0; 3], |swatch| {
            [swatch.rgba[0], swatch.rgba[1], swatch.rgba[2]]
        })
}
impl EditorView {
    pub(super) fn capture_palette_menu(&mut self) {
        self.palettes.menu = true;
        self.palettes.menu_index = 0;
    }

    pub(super) fn reconcile_palette_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.editor.palette_enabled() {
            let focused = self.palette_focus.is_focused(window)
                || self.palettes.field_focused(window, cx)
                || self.modal_focus.contains_focused(window, cx);
            self.palette_open = false;
            self.palettes.open = false;
            if focused {
                window.focus(&self.focus, cx);
            }
        }
    }
    pub(super) fn palette_name(&self) -> String {
        self.editor
            .palette_view()
            .map(|v| v.name)
            .unwrap_or_default()
    }
    pub(super) fn palette_index(&self) -> usize {
        self.editor
            .palette_view()
            .map(|v| v.preset)
            .unwrap_or_default()
    }
    pub(super) fn palette_choices(&self) -> Vec<voxelite_plugin_api::PaletteChoice> {
        self.editor
            .palette_view()
            .map(|v| v.choices.into_iter().filter(|c| c.read_only).collect())
            .unwrap_or_default()
    }
    pub(super) fn palette_request(
        &mut self,
        action: PaletteAction,
        cx: &mut Context<Self>,
    ) -> Option<PaletteOutcome> {
        match self.editor.palette_action(action) {
            Ok(outcome) => outcome,
            Err(error) => {
                self.report::<()>(Err(error), "", cx);
                None
            }
        }
    }
    fn manager_request(
        &mut self,
        action: PaletteAction,
        cx: &mut Context<Self>,
    ) -> Option<PaletteOutcome> {
        match self.editor.palette_action(action) {
            Ok(outcome) => {
                self.palettes.error = None;
                if let Some(outcome) = &outcome
                    && let Some(source) = &outcome.copied_from
                    && let Some(id) = outcome.state.custom
                {
                    self.palettes.copied_from = Some((PaletteId::Custom(id), source.clone()));
                    self.reveal_palette_choice();
                }
                cx.notify();
                outcome
            }
            Err(error) => {
                self.palettes.error = Some(error.to_string());
                cx.notify();
                None
            }
        }
    }
    fn sync_palette_fields(&mut self, preserve_hsb: bool, cx: &mut Context<Self>) {
        let Some(view) = self.editor.palette_view() else {
            return;
        };
        let rgb = selected_rgb(&view);
        let identity = view
            .palette
            .order
            .get(view.state.slot)
            .map(|slot| (view.selected, *slot));
        if identity != self.palettes.identity && view.state.preview.is_none() && !preserve_hsb {
            self.palettes.before = rgb;
        }
        self.palettes.identity = identity;
        if !preserve_hsb {
            self.palettes.hsb = Hsb::from_rgb(rgb, self.palettes.hsb.h);
        }
        let hsb = self.palettes.hsb;
        let values = [
            view.name,
            rgb_hex(rgb),
            rgb[0].to_string(),
            rgb[1].to_string(),
            rgb[2].to_string(),
            format!("{:.0}", hsb.h),
            format!("{:.0}", hsb.s * 100.),
            format!("{:.0}", hsb.b * 100.),
        ];
        for (field, text) in self.palettes.fields().into_iter().zip(&values) {
            field.update(cx, |field, cx| {
                if field.text() != text.as_str() {
                    field.set(text.clone(), cx);
                }
            });
        }
        self.palettes.synced = values;
    }
    pub(super) fn manage_palettes(&mut self, cx: &mut Context<Self>) {
        if self
            .palette_request(
                PaletteAction::Command(Op::ManagePalettes.command_id().into()),
                cx,
            )
            .is_some_and(|out| out.presentation == Some(PalettePresentation::Manage))
        {
            self.palettes.open = true;
            self.palettes.error = None;
            self.palettes.identity = None;
            self.palettes.menu = false;
            self.palettes.delete_confirmation = false;
            self.sync_palette_fields(false, cx);
        }
    }
    fn commit_palette_field(&mut self, index: usize, cx: &mut Context<Self>) -> bool {
        let text = self.palettes.fields()[index].read(cx).text().to_string();
        if text == self.palettes.synced[index] {
            return true;
        }
        let Some(view) = self.editor.palette_view() else {
            return false;
        };
        if index == 0 {
            if view.read_only {
                self.sync_palette_fields(false, cx);
                return true;
            }
            if self
                .manager_request(
                    PaletteAction::Edit {
                        operation: PaletteOperation::Rename,
                        name: text,
                        color: String::new(),
                    },
                    cx,
                )
                .is_none()
            {
                return false;
            }
        } else {
            let mut rgb = selected_rgb(&view);
            let mut hsb = self.palettes.hsb;
            if index == 1 {
                let Some(parsed) = parse_rgb_hex(&text) else {
                    self.palettes.error =
                        Some("Enter six hexadecimal digits, for example #A4C8E1.".into());
                    cx.notify();
                    return false;
                };
                rgb = parsed;
            } else {
                let max = if index < 5 {
                    255.
                } else if index == 5 {
                    360.
                } else {
                    100.
                };
                let value = text
                    .trim()
                    .trim_end_matches(['°', '%'])
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite() && *v >= 0. && *v <= max);
                let Some(value) = value else {
                    self.palettes.error = Some(format!("Enter a number from 0 to {max:.0}."));
                    cx.notify();
                    return false;
                };
                if index < 5 {
                    rgb[index - 2] = value.round() as u8;
                } else {
                    match index {
                        5 => hsb.h = value,
                        6 => hsb.s = value / 100.,
                        _ => hsb.b = value / 100.,
                    }
                    rgb = hsb.to_rgb();
                }
            }
            self.palettes.before = selected_rgb(&view);
            if self
                .manager_request(PaletteAction::SetColor { rgb, commit: true }, cx)
                .is_none()
            {
                return false;
            }
            self.palettes.hsb = if index >= 5 {
                hsb
            } else {
                Hsb::from_rgb(rgb, hsb.h)
            };
        }
        self.sync_palette_fields(true, cx);
        true
    }
    fn flush_palette_fields(&mut self, cx: &mut Context<Self>) -> bool {
        for index in 0..8 {
            if !self.commit_palette_field(index, cx) {
                return false;
            }
        }
        true
    }
    fn manager_action(&mut self, action: PaletteAction, cx: &mut Context<Self>) {
        if !self.flush_palette_fields(cx) {
            return;
        }
        if self
            .editor
            .palette_view()
            .is_some_and(|v| v.state.preview.is_some())
            && self
                .manager_request(PaletteAction::CommitColor, cx)
                .is_none()
        {
            return;
        }
        self.palettes.square_gesture.cancel();
        self.palettes.hue_gesture.cancel();
        self.palettes.preview_hsb = None;
        let replacing_color = matches!(
            action,
            PaletteAction::SetColor { .. }
                | PaletteAction::UseRecentColor {
                    target: voxelite_plugin_api::RecentColorTarget::SelectedSwatch,
                    ..
                }
        );
        let before = replacing_color
            .then(|| self.editor.palette_view().map(|v| selected_rgb(&v)))
            .flatten();
        if self.manager_request(action, cx).is_some() {
            self.palettes.delete_confirmation = false;
            self.sync_palette_fields(false, cx);
            if let Some(before) = before {
                self.palettes.before = before;
            }
            self.reveal_palette_choice();
        }
    }
    fn reveal_palette_choice(&self) {
        if let Some(view) = self.editor.palette_view()
            && let Some(index) = view.choices.iter().position(|c| c.id == view.selected)
        {
            // A new final row and the custom-only footer change the viewport height.
            // GPUI's item reveal uses the previous frame's bounds; bottom alignment
            // instead uses the newly laid out content, keeping the whole copy visible.
            if view.kind == voxelite_plugin_api::PaletteKind::Custom
                && index + 1 == view.choices.len()
            {
                self.palettes.sidebar_scroll.scroll_to_bottom();
                return;
            }
            let headers = match view.kind {
                voxelite_plugin_api::PaletteKind::Project => 1,
                voxelite_plugin_api::PaletteKind::BuiltIn => 2,
                voxelite_plugin_api::PaletteKind::Custom => 3,
            };
            self.palettes.sidebar_scroll.scroll_to_item(index + headers);
        }
    }
    fn palette_edit(&mut self, operation: PaletteOperation, cx: &mut Context<Self>) {
        let name = if operation == PaletteOperation::Create {
            "Untitled palette".into()
        } else {
            self.palette_name()
        };
        self.manager_action(
            PaletteAction::Edit {
                operation,
                name,
                color: rgb_hex(self.palettes.hsb.to_rgb()),
            },
            cx,
        );
    }
    fn picker_change(&mut self, change: &ColorChange, cx: &mut Context<Self>) {
        if !self.flush_palette_fields(cx) {
            return;
        }
        let Some(view) = self.editor.palette_view() else {
            return;
        };
        if view.state.preview.is_none() {
            self.palettes.before = selected_rgb(&view);
            self.palettes.preview_hsb = Some(self.palettes.hsb);
        }
        if self
            .manager_request(
                PaletteAction::SetColor {
                    rgb: change.color.to_rgb(),
                    commit: change.commit,
                },
                cx,
            )
            .is_some()
        {
            self.palettes.hsb = change.color;
            self.sync_palette_fields(true, cx);
            if change.commit {
                self.palettes.preview_hsb = None;
            }
        }
    }
    fn cancel_palette_color(&mut self, cx: &mut Context<Self>) {
        self.palettes.square_gesture.cancel();
        self.palettes.hue_gesture.cancel();
        self.manager_request(PaletteAction::CancelColor, cx);
        let restore_hsb = self.palettes.preview_hsb.take();
        if let Some(hsb) = restore_hsb {
            self.palettes.hsb = hsb;
        }
        self.sync_palette_fields(restore_hsb.is_some(), cx);
        if self.editor.palette_view().is_some_and(|v| v.read_only) {
            self.palettes.copied_from = None;
        }
        self.reveal_palette_choice();
    }
    pub(super) fn close_palettes(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.flush_palette_fields(cx) {
            return;
        }
        if self
            .manager_request(PaletteAction::CommitColor, cx)
            .is_none()
        {
            return;
        }
        self.palettes.open = false;
        self.palettes.square_gesture.cancel();
        self.palettes.hue_gesture.cancel();
        self.palettes.menu = false;
        window.focus(&self.focus, cx);
        cx.notify();
    }
    /// Capture Enter before the embedded text editor consumes it; leave editing keys alone.
    pub(super) fn palette_dialog_key(
        &mut self,
        e: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.palettes.open {
            return;
        }
        let key = e.keystroke.key.as_str();
        if key == "escape" {
            if cx.has_active_drag() {
                cx.stop_active_drag(window);
                self.palettes.drop_boundary = None;
            } else if self.palettes.menu {
                self.palettes.menu = false;
            } else if self.palettes.delete_confirmation {
                self.palettes.delete_confirmation = false;
            } else if self
                .editor
                .palette_view()
                .is_some_and(|v| v.state.preview.is_some())
            {
                self.cancel_palette_color(cx);
                window.focus(&self.palettes.grid_focus, cx);
            } else if self
                .palettes
                .fields()
                .iter()
                .enumerate()
                .any(|(i, f)| f.read(cx).text() != self.palettes.synced[i])
            {
                self.sync_palette_fields(false, cx);
                self.palettes.error = None;
                window.focus(&self.palettes.grid_focus, cx);
            } else {
                self.close_palettes(window, cx);
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if key == "tab" {
            // Tab leaves the menu; arrow keys own its highlighted entry.
            self.palettes.menu = false;
            Dialog::trap_tab(&self.modal_focus, e.keystroke.modifiers.shift, window, cx);
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.palettes.menu {
            let count = self
                .editor
                .palette_view()
                .map_or(1, |v| v.formats.len() + 1);
            match key {
                "up" => self.palettes.menu_index = (self.palettes.menu_index + count - 1) % count,
                "down" => self.palettes.menu_index = (self.palettes.menu_index + 1) % count,
                "enter" | "space" => self.palette_menu_action(self.palettes.menu_index, cx),
                _ => return,
            }
            cx.stop_propagation();
            cx.notify();
            return;
        }
        if self.palettes.field_focused(window, cx) && key == "enter" {
            for (i, field) in self.palettes.fields().iter().enumerate() {
                if field.read(cx).focused(window) {
                    self.commit_palette_field(i, cx);
                    break;
                }
            }
            cx.stop_propagation();
        }
    }
}

impl EditorView {
    pub(super) fn control(
        &self,
        id: impl Into<ElementId>,
        name: &'static str,
        text: impl Into<SharedString>,
        cx: &Context<Self>,
        callback: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        self.styled_control(id, name, text, ButtonStyle::Ghost, cx, callback)
    }
    pub(super) fn styled_control(
        &self,
        id: impl Into<ElementId>,
        name: &'static str,
        text: impl Into<SharedString>,
        style: ButtonStyle,
        cx: &Context<Self>,
        callback: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Stateful<Div> {
        let t = self.theme;
        let mut button = Button::new(id, text, t).legacy_style(style);
        if !name.is_empty() {
            button = button.icon(name);
        }
        button
            .build()
            .border_1()
            .border_color(t.line())
            .on_click(cx.listener(move |this, _, window, cx| {
                window.focus(&this.focus, cx);
                callback(this, window, cx)
            }))
    }
}
