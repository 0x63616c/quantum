//! Unsaved-change confirmation and deferred commands, independent of a window.
//!
//! The dialog gate also covers open/import/export operations. Unrelated dialogs
//! must not consume a command deferred until a project save succeeds; choosing
//! an export instead of a project save cancels that deferred command.
use super::*;

#[cfg(not(target_family = "wasm"))]
mod format;
#[cfg(not(target_family = "wasm"))]
use format::SaveFormat;
#[cfg(target_os = "macos")]
mod macos;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SaveChoice {
    DontSave = 0,
    Cancel = 1,
    Save = 2,
}
impl SaveChoice {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::DontSave,
            2 => Self::Save,
            _ => Self::Cancel,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingAction {
    Run(Op),
    Save,
    Cancelled,
}

pub(super) enum CloseRequest {
    Allow,
    Blocked,
    Confirm,
}

pub(super) struct SaveFlow {
    pending: Option<Op>,
    choice: SaveChoice,
    after_save: Option<Op>,
    dialog_open: bool,
}

impl Default for SaveFlow {
    fn default() -> Self {
        Self {
            pending: None,
            choice: SaveChoice::Save,
            after_save: None,
            dialog_open: false,
        }
    }
}

impl SaveFlow {
    pub(super) fn needs_confirmation(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn choice_index(&self) -> usize {
        self.choice as usize
    }
    pub(super) fn dialog_open(&self) -> bool {
        self.dialog_open
    }

    pub(super) fn confirm_if_dirty(&mut self, op: Op, dirty: impl FnOnce() -> bool) -> bool {
        if matches!(op, Op::New | Op::Open | Op::Quit) && dirty() {
            self.pending = Some(op);
            self.choice = SaveChoice::Save;
            true
        } else {
            false
        }
    }

    pub(super) fn request_close(&mut self, dirty: impl FnOnce() -> bool) -> CloseRequest {
        if self.dialog_open {
            CloseRequest::Blocked
        } else if self.confirm_if_dirty(Op::Quit, dirty) {
            CloseRequest::Confirm
        } else {
            CloseRequest::Allow
        }
    }

    pub(super) fn cancel_confirmation(&mut self) {
        self.pending = None;
    }
    pub(super) fn begin_dialog(&mut self) {
        self.dialog_open = true;
    }
    pub(super) fn finish_dialog(&mut self) {
        self.dialog_open = false;
    }

    #[cfg(not(target_family = "wasm"))]
    fn cancel_save(&mut self) {
        self.finish_dialog();
        self.after_save = None;
    }

    // Native saves end the dialog gate separately. Browser downloads are
    // synchronous and never open it; both consume the continuation on failure.
    pub(super) fn complete_save(&mut self, success: bool) -> Option<Op> {
        let next = self.after_save.take();
        next.filter(|_| success)
    }

    fn resolve(&mut self, choice: SaveChoice) -> Option<PendingAction> {
        let op = self.pending.take()?;
        Some(match choice {
            SaveChoice::DontSave => PendingAction::Run(op),
            SaveChoice::Save => {
                self.after_save = Some(op);
                PendingAction::Save
            }
            SaveChoice::Cancel => PendingAction::Cancelled,
        })
    }

    fn key(&mut self, stroke: &Keystroke, platform: ShortcutPlatform) -> Option<SaveChoice> {
        let key = navigation_key(stroke).unwrap_or_default();
        if stroke.key == "d" && platform.is_primary(stroke.modifiers) {
            return Some(SaveChoice::DontSave);
        }
        match key {
            "escape" => return Some(SaveChoice::Cancel),
            "left" => self.choice = SaveChoice::from_index((self.choice_index() + 2) % 3),
            "right" | "tab" => {
                self.choice = SaveChoice::from_index(
                    (self.choice_index() + if stroke.modifiers.shift { 2 } else { 1 }) % 3,
                );
            }
            "enter" => return Some(self.choice),
            _ => {}
        }
        None
    }
}

impl EditorView {
    #[cfg(not(target_family = "wasm"))]
    pub(super) fn open_dialog(&mut self, cx: &mut Context<Self>) {
        self.save_flow.begin_dialog();
        self.status = "Choose a project to open…".into();
        self.drag = None;
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open .voxl project".into()),
        });
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(Some(paths))) if !paths.is_empty() => {
                        let path = paths[0].clone();
                        this.status = "Opening project…".into();
                        let task = cx.background_executor().spawn(async move { Editor::open(&path) });
                        cx.spawn(async move |this, cx| {
                            let result = task.await;
                            let _ = this.update(cx, |this, cx| {
                                this.save_flow.finish_dialog();
                                match result {
                                    Ok(editor) => { this.replace_editor(editor, cx); this.status = "Project opened".into(); }
                                    Err(error) => { this.status = error.to_string().into(); this.error = true; }
                                }
                                cx.notify();
                            });
                        }).detach();
                    }
                    Ok(Ok(_)) => { this.save_flow.finish_dialog(); }
                    _ => {
                        this.save_flow.finish_dialog();
                        this.status = "Open dialog unavailable. Enable a desktop file portal or open a .voxl path from the command line.".into();
                        this.error = true;
                    }
                }
                cx.notify();
            });
        }).detach();
    }
    #[cfg(not(target_family = "wasm"))]
    fn write_project(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.save_flow.begin_dialog();
        self.status = "Saving project…".into();
        let save = self.editor.prepare_save(&path);
        let task = cx.background_executor().spawn(async move { save.write() });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.save_flow.finish_dialog();
                let next = this.save_flow.complete_save(result.is_ok());
                match result {
                    Ok(saved) => {
                        this.editor.finish_save(saved);
                        this.status = "Saved".into();
                        this.error = false;
                        if let Some(op) = next {
                            this.run(op, cx);
                        }
                    }
                    Err(error) => {
                        this.status = error.to_string().into();
                        this.error = true;
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
    #[cfg(not(target_family = "wasm"))]
    fn write_export(&mut self, path: PathBuf, format: SaveFormat, cx: &mut Context<Self>) {
        // An export cannot satisfy a deferred New/Open/Quit that needs a project save.
        self.save_flow.complete_save(false);
        self.save_flow.begin_dialog();
        self.status = format!("Exporting {}…", format.label()).into();
        let snapshot = self.editor.snapshot().clone();
        let options = RenderOptions {
            width: 1024,
            height: 768,
            view: View::Camera(self.editor.camera.camera()),
            background: if self.theme == Theme::Dark {
                [0, 0, 0, 255]
            } else {
                [255; 4]
            },
            ..Default::default()
        };
        let task = cx.background_executor().spawn(async move {
            let bytes = match format {
                SaveFormat::Glb => voxelite_export::glb(&snapshot)?,
                SaveFormat::Png => voxelite_export::render_png(&snapshot, &options)?,
                SaveFormat::Project => anyhow::bail!("Choose GLB or PNG for export"),
            };
            std::fs::write(&path, bytes)?;
            Ok::<_, anyhow::Error>(path)
        });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.save_flow.finish_dialog();
                match result {
                    Ok(path) => {
                        this.status = format!("Exported {}", path.display()).into();
                        this.error = false;
                    }
                    Err(error) => {
                        this.status = error.to_string().into();
                        this.error = true;
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
    #[cfg(not(target_family = "wasm"))]
    pub(super) fn save(&mut self, save_as: bool, cx: &mut Context<Self>) {
        if !save_as && let Some(path) = self.editor.path.clone() {
            self.write_project(path, cx);
            return;
        }
        self.choose_save_path(cx);
    }
    #[cfg(not(target_family = "wasm"))]
    pub(super) fn choose_save_path(&mut self, cx: &mut Context<Self>) {
        let directory = self
            .editor
            .path
            .as_ref()
            .and_then(|p| p.parent())
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let suggested = format!(
            "{}.voxl",
            self.editor.snapshot().name.replace(['/', '\\'], "-")
        );
        self.save_flow.begin_dialog();
        self.status = "Choose where to save this project…".into();
        #[cfg(target_os = "macos")]
        let receiver = macos::choose_save_path(&directory, &suggested, cx);
        #[cfg(not(target_os = "macos"))]
        let receiver = cx.prompt_for_new_path(&directory, Some(&suggested));
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            #[cfg(not(target_os = "macos"))]
            let result = result.map(|result| {
                result.map(|path| path.map(|mut path| {
                    let format = SaveFormat::from_extension(
                        path.extension().and_then(|ext| ext.to_str()).unwrap_or_default(),
                    );
                    if path.extension().is_none() { path.set_extension(format.extension()); }
                    (path, format)
                }))
            });
            let _ = this.update(cx, |this, cx| {
                match result {
                    Ok(Ok(Some((path, format)))) => {
                        match format {
                            SaveFormat::Project => this.write_project(path, cx),
                            SaveFormat::Glb | SaveFormat::Png => this.write_export(path, format, cx),
                        }
                    }
                    Ok(Ok(None)) => { this.save_flow.cancel_save(); }
                    _ => {
                        this.save_flow.cancel_save();
                        this.status = if cfg!(target_os = "macos") {
                            "Save dialog unavailable; edits are still open."
                        } else {
                            "Save dialog unavailable. Enable a desktop file portal; edits are still open."
                        }.into();
                        this.error = true;
                    }
                }
                cx.notify();
            });
        }).detach();
    }
    pub(super) fn resolve_pending(&mut self, choice: usize, cx: &mut Context<Self>) {
        let Some(action) = self.save_flow.resolve(SaveChoice::from_index(choice)) else {
            return;
        };
        match action {
            PendingAction::Run(op) => self.run(op, cx),
            PendingAction::Save => self.save(false, cx),
            PendingAction::Cancelled => {}
        }
        cx.notify();
    }

    pub(super) fn pending_key(&mut self, stroke: &Keystroke, cx: &mut Context<Self>) {
        if let Some(choice) = self.save_flow.key(stroke, ShortcutPlatform::current()) {
            self.resolve_pending(choice as usize, cx);
        }
        cx.stop_propagation();
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::{CloseRequest, Op, PendingAction, SaveChoice, SaveFlow, ShortcutPlatform};
    use gpui::Keystroke;

    fn key(flow: &mut SaveFlow, stroke: &str) -> Option<SaveChoice> {
        flow.key(
            &Keystroke::parse(stroke).unwrap(),
            ShortcutPlatform::current(),
        )
    }

    #[test]
    fn only_dirty_destructive_commands_require_confirmation_and_default_to_save() {
        let mut flow = SaveFlow::default();
        for op in [Op::New, Op::Open, Op::Quit] {
            assert!(!flow.confirm_if_dirty(op, || false));
            assert!(!flow.needs_confirmation());
            assert!(flow.confirm_if_dirty(op, || true));
            assert_eq!(flow.choice_index(), 2);
            key(&mut flow, "left");
            assert_eq!(flow.choice_index(), 1);
            assert!(flow.confirm_if_dirty(op, || true));
            assert_eq!(flow.choice_index(), 2);
            flow.cancel_confirmation();
        }
        for op in [Op::Save, Op::SaveAs, Op::Undo, Op::Commands] {
            assert!(!flow.confirm_if_dirty(op, || true));
            assert!(!flow.needs_confirmation());
        }
    }

    #[test]
    fn dont_save_runs_once_cancel_does_nothing_and_save_defers_each_operation() {
        for op in [Op::New, Op::Open, Op::Quit] {
            for choice in [SaveChoice::DontSave, SaveChoice::Cancel, SaveChoice::Save] {
                let mut flow = SaveFlow::default();
                flow.confirm_if_dirty(op, || true);
                let expected = match choice {
                    SaveChoice::DontSave => PendingAction::Run(op),
                    SaveChoice::Cancel => PendingAction::Cancelled,
                    SaveChoice::Save => PendingAction::Save,
                };
                assert_eq!(flow.resolve(choice), Some(expected));
                assert!(!flow.needs_confirmation());
                assert_eq!(flow.resolve(choice), None);
                assert_eq!(
                    flow.complete_save(true),
                    (choice == SaveChoice::Save).then_some(op)
                );
                assert_eq!(flow.complete_save(true), None);
            }
        }
    }

    #[test]
    fn failed_save_consumes_continuation_and_keeps_later_save_from_running_it() {
        let mut flow = SaveFlow::default();
        flow.confirm_if_dirty(Op::Quit, || true);
        assert_eq!(flow.resolve(SaveChoice::Save), Some(PendingAction::Save));
        flow.begin_dialog();
        assert!(flow.dialog_open());
        flow.finish_dialog();
        assert_eq!(flow.complete_save(false), None);
        assert!(!flow.dialog_open());
        assert!(!flow.needs_confirmation());
        flow.begin_dialog();
        flow.finish_dialog();
        assert_eq!(flow.complete_save(true), None);
    }

    #[test]
    #[cfg(not(target_family = "wasm"))]
    fn cancelling_or_failing_path_chooser_aborts_the_entire_deferred_operation() {
        for op in [Op::New, Op::Open, Op::Quit] {
            let mut flow = SaveFlow::default();
            flow.confirm_if_dirty(op, || true);
            flow.resolve(SaveChoice::Save);
            flow.begin_dialog();
            flow.cancel_save();
            assert!(!flow.dialog_open());
            assert!(!flow.needs_confirmation());
            assert_eq!(flow.complete_save(true), None);
        }
    }

    #[test]
    fn path_selection_to_write_keeps_dialog_gate_and_continuation_until_completion() {
        let mut flow = SaveFlow::default();
        flow.confirm_if_dirty(Op::Open, || true);
        flow.resolve(SaveChoice::Save);
        flow.begin_dialog(); // Path chooser.
        flow.begin_dialog(); // Selected path starts the write without unblocking.
        assert!(flow.dialog_open());
        assert!(matches!(flow.request_close(|| true), CloseRequest::Blocked));
        flow.finish_dialog();
        assert_eq!(flow.complete_save(true), Some(Op::Open));
        // The continued Open operation starts its own dialog.
        flow.begin_dialog();
        assert!(matches!(
            flow.request_close(|| false),
            CloseRequest::Blocked
        ));
        flow.finish_dialog();
        assert_eq!(flow.complete_save(true), None);
    }

    #[test]
    fn close_obeys_dialog_gate_then_dirty_state_and_preserves_existing_pending_state() {
        let mut flow = SaveFlow::default();
        assert!(matches!(flow.request_close(|| false), CloseRequest::Allow));
        assert!(matches!(flow.request_close(|| true), CloseRequest::Confirm));
        assert_eq!(flow.choice_index(), 2);
        flow.begin_dialog();
        assert!(matches!(
            flow.request_close(|| false),
            CloseRequest::Blocked
        ));
        assert!(matches!(flow.request_close(|| true), CloseRequest::Blocked));
        assert_eq!(flow.pending, Some(Op::Quit));
        flow.finish_dialog();
        // As before, a clean close allows closing without rewriting a pending op.
        assert!(matches!(flow.request_close(|| false), CloseRequest::Allow));
        assert_eq!(flow.pending, Some(Op::Quit));
        flow.cancel_confirmation();
        assert_eq!(flow.resolve(SaveChoice::Save), None);
    }

    #[test]
    fn confirmation_navigation_wraps_and_enter_escape_have_the_original_outcomes() {
        let mut flow = SaveFlow::default();
        flow.confirm_if_dirty(Op::New, || true);
        assert_eq!(key(&mut flow, "enter"), Some(SaveChoice::Save));
        key(&mut flow, "right");
        assert_eq!(key(&mut flow, "enter"), Some(SaveChoice::DontSave));
        key(&mut flow, "tab");
        assert_eq!(key(&mut flow, "enter"), Some(SaveChoice::Cancel));
        key(&mut flow, "shift-tab");
        key(&mut flow, "left");
        assert_eq!(key(&mut flow, "enter"), Some(SaveChoice::Save));
        assert_eq!(key(&mut flow, "escape"), Some(SaveChoice::Cancel));
        assert_eq!(key(&mut flow, "shift-right"), None);
        assert_eq!(flow.choice_index(), 2);
        assert_eq!(key(&mut flow, "shift-enter"), None);
    }

    #[test]
    fn dont_save_shortcut_requires_exact_host_primary_modifier() {
        for (platform, primary, wrong) in [
            (ShortcutPlatform::Apple, "cmd-d", "ctrl-d"),
            (ShortcutPlatform::WindowsLinux, "ctrl-d", "cmd-d"),
        ] {
            let mut flow = SaveFlow::default();
            flow.confirm_if_dirty(Op::Quit, || true);
            assert_eq!(
                flow.key(&Keystroke::parse(primary).unwrap(), platform),
                Some(SaveChoice::DontSave)
            );
            for stroke in [wrong, "d", "ctrl-alt-d", "cmd-shift-d"] {
                assert_eq!(flow.key(&Keystroke::parse(stroke).unwrap(), platform), None);
            }
        }
    }

    #[test]
    fn unrelated_dialog_completion_and_confirmation_dismissal_do_not_consume_save() {
        let mut flow = SaveFlow::default();
        flow.confirm_if_dirty(Op::New, || true);
        flow.resolve(SaveChoice::Save);
        flow.begin_dialog();
        flow.finish_dialog();
        flow.cancel_confirmation();
        assert_eq!(flow.complete_save(true), Some(Op::New));
        // Browser save completion does not mutate an independently held gate.
        flow.begin_dialog();
        assert_eq!(flow.complete_save(false), None);
        assert!(flow.dialog_open());
    }
}
