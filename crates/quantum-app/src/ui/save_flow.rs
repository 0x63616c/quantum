//! Unsaved-change confirmation and deferred commands, independent of a window.
//!
//! The dialog gate also covers open/import/export operations. Unrelated dialogs
//! must not consume a command deferred until a project save succeeds; choosing
//! an export instead of a project save cancels that deferred command.
use super::*;
use quantum_ui::Space;

use quantum_app::save_format::SaveFormat;
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
    working: bool,
    completion: Option<(String, std::time::Instant)>,
    open_path: Option<PathBuf>,
    // Deferred open target, not the currently active pathless sample session.
    sample: Option<quantum_app::samples::Sample>,
    pub(super) picker: quantum_app::save_picker::SavePicker,
    before_picker: Option<(HintText, bool)>,
}

impl Default for SaveFlow {
    fn default() -> Self {
        Self {
            pending: None,
            choice: SaveChoice::Save,
            after_save: None,
            dialog_open: false,
            working: false,
            completion: None,
            open_path: None,
            sample: None,
            picker: Default::default(),
            before_picker: None,
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
        {
            self.open_path = None;
            self.sample = None;
        }
    }
    pub(super) fn begin_dialog(&mut self) {
        self.dialog_open = true;
    }
    pub(super) fn finish_dialog(&mut self) {
        self.dialog_open = false;
        self.working = false;
    }

    pub(super) fn working(&self) -> bool {
        self.working
    }

    pub(super) fn completion(&self, message: &str) -> Option<std::time::Instant> {
        self.completion
            .as_ref()
            .filter(|(text, _)| text == message)
            .map(|(_, at)| *at)
    }

    fn begin_work(&mut self) {
        self.begin_dialog();
        self.working = true;
        self.completion = None;
    }

    fn finished_work(&mut self, status: &HintText) {
        self.completion = status
            .plain_text()
            .map(|text| (text.to_string(), std::time::Instant::now()));
    }

    fn cancel_save(&mut self) {
        self.picker.cancel();
        self.finish_dialog();
        self.after_save = None;
        self.open_path = None;
        self.sample = None;
    }

    fn restore_picker_status(&mut self, status: &mut HintText, error: &mut bool) {
        if let Some(previous) = self.before_picker.take() {
            (*status, *error) = previous;
        }
    }

    fn cancel_picker(&mut self, status: &mut HintText, error: &mut bool) -> bool {
        if !self.picker.waiting() {
            return false;
        }
        self.cancel_save();
        self.restore_picker_status(status, error);
        true
    }

    fn complete_picker(
        &mut self,
        request: u64,
        result: anyhow::Result<Option<(PathBuf, SaveFormat)>>,
        status: &mut HintText,
        error: &mut bool,
    ) -> Option<(PathBuf, SaveFormat)> {
        if !self.picker.complete(request) {
            return None;
        }
        self.restore_picker_status(status, error);
        match result {
            Ok(Some((path, format))) if !path.as_os_str().is_empty() => Some((path, format)),
            Ok(_) => {
                self.cancel_save();
                None
            }
            Err(_) => {
                self.cancel_save();
                *status = if cfg!(target_os = "macos") {
                    "Save dialog unavailable; edits are still open."
                } else {
                    "Save dialog unavailable. Enable a desktop file portal; edits are still open."
                }
                .into();
                *error = true;
                None
            }
        }
    }

    // Saving ends the dialog gate separately and consumes the continuation
    // on failure.
    pub(super) fn complete_save(&mut self, success: bool) -> Option<Op> {
        let next = self.after_save.take();
        if !success {
            self.open_path = None;
            self.sample = None;
        }
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
            SaveChoice::Cancel => {
                {
                    self.open_path = None;
                    self.sample = None;
                }
                PendingAction::Cancelled
            }
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

#[cfg(test)]
mod loading_tests {
    use super::SaveFlow;

    #[test]
    fn only_io_work_shimmers_and_completion_does_not_animate_later_status_messages() {
        let mut flow = SaveFlow::default();
        flow.begin_dialog();
        assert!(!flow.working());
        flow.begin_work();
        assert!(flow.working());
        flow.finish_dialog();
        flow.finished_work(&"Saved".into());
        assert!(!flow.working());
        assert!(flow.completion("Saved").is_some());
        assert!(flow.completion("Voxel placed").is_none());
        flow.begin_work();
        assert!(flow.completion("Saved").is_none());
        flow.finish_dialog(); // Includes cancellation and failure paths.
        assert!(!flow.working());
    }
}

impl EditorView {
    pub(super) fn open_recent(
        &mut self,
        path: PathBuf,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.enabled(Op::Open) {
            return;
        }
        self.pause_history(cx);
        self.menu = None;
        self.commands.close();
        self.palette_open = false;
        self.save_flow.sample = None;
        self.save_flow.open_path = Some(path);
        if !self
            .save_flow
            .confirm_if_dirty(Op::Open, || self.editor.needs_save())
        {
            self.open_dialog(cx);
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    pub(super) fn open_sample(
        &mut self,
        sample: quantum_app::samples::Sample,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.enabled(Op::Open) {
            return;
        }
        self.pause_history(cx);
        self.menu = None;
        self.commands.close();
        self.palette_open = false;
        self.save_flow.open_path = None;
        self.save_flow.sample = Some(sample);
        if !self
            .save_flow
            .confirm_if_dirty(Op::Open, || self.editor.needs_save())
        {
            self.open_dialog(cx);
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn load_project(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.save_flow.begin_work();
        self.error = false;
        self.status = "Opening project…".into();
        let task = cx
            .background_executor()
            .spawn(async move { Editor::load_session(&path) });
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |this, cx| {
                this.save_flow.finish_dialog();
                match result.and_then(Editor::from_session) {
                    Ok(editor) => {
                        this.replace_editor(editor, cx);
                        this.status = "Project opened".into();
                    }
                    Err(error) => {
                        this.status = error.to_string().into();
                        this.error = true;
                    }
                }
                this.save_flow.finished_work(&this.status);
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn open_dialog(&mut self, cx: &mut Context<Self>) {
        if let Some(sample) = self.save_flow.sample.take() {
            self.save_flow.begin_dialog();
            let task = cx
                .background_executor()
                .spawn(async move { sample.session() });
            cx.spawn(async move |this, cx| {
                let result = task.await;
                let _ = this.update(cx, |this, cx| {
                    this.save_flow.finish_dialog();
                    match result.and_then(Editor::from_session) {
                        Ok(editor) => this.replace_editor(editor, cx),
                        Err(error) => {
                            this.status = error.to_string().into();
                            this.error = true;
                        }
                    }
                    cx.notify();
                });
            })
            .detach();
            return;
        }
        if let Some(path) = self.save_flow.open_path.take() {
            self.load_project(path, cx);
            return;
        }
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
                        this.load_project(paths[0].clone(), cx);
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
    fn write_project(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.save_flow.begin_work();
        self.error = false;
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
                        welcome::record(&this.editor, cx);
                        this.status = "Saved".into();
                        this.error = false;
                        if let Some(op) = next {
                            this.save_flow.begin_dialog();
                            welcome::after_record(op, cx);
                        }
                    }
                    Err(error) => {
                        this.status = error.to_string().into();
                        this.error = true;
                    }
                }
                this.save_flow.finished_work(&this.status);
                cx.notify();
            });
        })
        .detach();
    }
    fn write_export(
        &mut self,
        path: PathBuf,
        format: quantum_plugin_api::FormatContribution,
        cx: &mut Context<Self>,
    ) {
        // An export cannot satisfy a deferred New/Open/Quit that needs a project save.
        self.save_flow.complete_save(false);
        self.save_flow.begin_work();
        self.error = false;
        self.status = format!("Exporting {}…", format.label_key).into();
        let job = match self.editor.prepare_export(&format.id) {
            Ok(job) => job,
            Err(error) => {
                self.save_flow.finish_dialog();
                self.report::<()>(Err(error), "", cx);
                return;
            }
        };
        let snapshot = self.editor.snapshot().clone();
        let options = RenderOptions {
            passes: self.editor.render_plan(),
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
        let executor = cx.background_executor().clone();
        let task = executor.spawn(async move {
            job.run(&quantum_plugin_host::SnapshotExport {
                snapshot: &snapshot,
                render_view: &quantum_app::editor::render_view(&options),
            })
        });
        cx.spawn(async move |this, cx| {
            let completion = task.await;
            let bytes = this.update(cx, |this, _| this.editor.finish_export(completion));
            let result: anyhow::Result<PathBuf> = async {
                let bytes = bytes??;
                executor
                    .spawn(async move {
                        quantum_format::storage::safe_write(&path, bytes)?;
                        Ok::<_, anyhow::Error>(path)
                    })
                    .await
            }
            .await;
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
                this.save_flow.finished_work(&this.status);
                cx.notify();
            });
        })
        .detach();
    }
    pub(super) fn save(&mut self, save_as: bool, cx: &mut Context<Self>) {
        if !save_as && let Some(path) = self.editor.path.clone() {
            self.write_project(path, cx);
            return;
        }
        self.choose_save_path(cx);
    }
    pub(super) fn choose_save_path(&mut self, cx: &mut Context<Self>) {
        let directory = match self.editor.attachment_root() {
            Ok(directory) => directory,
            Err(error) => {
                self.status = error.to_string().into();
                self.error = true;
                cx.notify();
                return;
            }
        };
        if let Err(error) = std::fs::create_dir_all(&directory) {
            self.status = error.to_string().into();
            self.error = true;
            cx.notify();
            return;
        }
        let suggested = format!(
            "{}.voxl",
            self.editor.snapshot().name.replace(['/', '\\'], "-")
        );
        let request = self.save_flow.picker.begin();
        self.save_flow.before_picker = Some((self.status.clone(), self.error));
        self.save_flow.begin_dialog();
        self.status = "Waiting for file dialog…".into();
        let formats = SaveFormat::available(&self.editor.plugin_host());
        #[cfg(target_os = "macos")]
        let receiver = macos::choose_save_path(&directory, &suggested, formats, cx);
        #[cfg(not(target_os = "macos"))]
        let receiver = cx.prompt_for_new_path(&directory, Some(&suggested));
        cx.spawn(async move |this, cx| {
            let result = receiver.await;
            #[cfg(not(target_os = "macos"))]
            let result = result.map(|result| {
                result.map(|path| {
                    path.map(|path| {
                        let format = SaveFormat::from_extension(
                            &formats,
                            path.extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or_default(),
                        );
                        (path, format)
                    })
                })
            });
            let _ = this.update(cx, |this, cx| {
                let result = result
                    .map_err(anyhow::Error::from)
                    .and_then(|result| result);
                if let Some((path, format)) = this.save_flow.complete_picker(
                    request,
                    result,
                    &mut this.status,
                    &mut this.error,
                ) {
                    #[cfg(not(target_os = "macos"))]
                    let path = {
                        let mut path = path;
                        if path.extension().is_none() {
                            path.set_extension(format.extension());
                        }
                        path
                    };
                    match format {
                        SaveFormat::Project => this.write_project(path, cx),
                        SaveFormat::Export(format) => this.write_export(path, format, cx),
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
    pub(super) fn cancel_save_picker(&mut self, cx: &mut Context<Self>) {
        if self
            .save_flow
            .cancel_picker(&mut self.status, &mut self.error)
        {
            cx.notify();
        }
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
mod tests;

impl EditorView {
    pub(super) fn working_dialog(&self, cx: &Context<Self>) -> AnyElement {
        let t = self.theme;
        div()
            .absolute()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(t.app_scrim())
            .child(
                self.floating()
                    .p(px(Space::S6.px()))
                    .child(
                        row()
                            .gap(px(Space::S3.px()))
                            .when(self.save_flow.picker.waiting(), |d| {
                                let reduced = quantum_app::reduce_motion::prefers_reduced_motion();
                                d.child(
                                    quantum_ui::Spinner::new("save-picker-spinner", t)
                                        .reduced_motion(reduced || self.capture_state.is_some()),
                                )
                            })
                            .child(
                                if self.save_flow.working()
                                    && let Some(text) = self.status.plain_text()
                                {
                                    quantum_ui::AsciiReveal::new(
                                        "io-dialog-progress",
                                        text.to_owned(),
                                        t,
                                    )
                                    .loading(true)
                                    .reduced_motion(self.reduced_motion)
                                    .into_any_element()
                                } else {
                                    self.status.clone().into_any_element()
                                },
                            ),
                    )
                    .when(self.save_flow.picker.waiting(), |d| {
                        d.child(
                            row().mt(px(Space::S5.px())).child(
                                Button::design("cancel-save-picker", "Cancel", t)
                                    .build()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.cancel_save_picker(cx);
                                        window.focus(&this.focus, cx);
                                    })),
                            ),
                        )
                    }),
            )
            .into_any_element()
    }
}
