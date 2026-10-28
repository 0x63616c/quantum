//! Editor construction, window observers and document replacement.
use super::*;

impl EditorView {
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: declared boolean grid setting"
    )]
    pub fn new(
        mut editor: Editor,
        override_theme: Option<Theme>,
        socket: Option<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        settings::init(override_theme, cx);
        {
            welcome::record(&editor, cx);
            welcome::observe_editor(cx);
        }
        let preferences = cx.global::<settings::SettingsState>().values.clone();
        let theme = settings::theme(preferences.appearance, window);
        editor
            .set_grid(preferences.grid)
            .expect("declared boolean grid setting");
        cx.observe_global_in::<settings::SettingsState>(window, |this, window, cx| {
            this.sync_preferences(window, cx)
        })
        .detach();
        cx.observe_window_appearance(window, |this, window, cx| this.sync_preferences(window, cx))
            .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self::observe_window(window, cx);
        let command_input = Self::command_input(window, cx);
        let palettes = palettes::PaletteManager::new(window, cx);
        let library_error = editor
            .load_palette_library()
            .err()
            .map(|error| error.to_string());
        let agent = agent::Agent::new(&editor, socket);
        let agent_error = agent.as_ref().err().map(ToString::to_string);
        let agent = agent.unwrap_or_default();
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(80))
                    .await;
                if this.update(cx, |this, cx| this.poll_agent(cx)).is_err() {
                    break;
                }
            }
        })
        .detach();
        #[cfg(target_os = "macos")]
        Self::monitor_gpu(cx);
        Self {
            capture_state: None,
            overlays: quantum_ui::OverlayState::new(cx),
            reduced_motion: quantum_app::reduce_motion::prefers_reduced_motion(),
            agent,
            palettes,
            discussion: comments::Discussion::new(window, cx),
            panel: SidebarTab::Session,
            timeline: history::Timeline::new(window, cx),
            cursor: editor.snapshot().bounds.min,
            camera_view: Some(Op::Frame),
            camera_fitted: true,
            camera_anim: quantum_render::CameraAnimator::new(),
            anim_epoch: std::time::Instant::now(),
            zoom_reference_span: editor.camera.span,
            viewport: viewport::Viewport::new(editor.camera),
            editor,
            preferences,
            settings_window: None,
            palette_focus: cx.focus_handle(),
            modal_focus: cx.focus_handle(),
            sidebar_scroll: ScrollHandle::new(),
            theme,
            focus,
            last_title: String::new(),
            last_edited: None,
            hover: None,
            drag: None,
            overlay_drag: None,
            overlay_hover: None,
            overlay_bounds: Rc::default(),
            menu: None,
            menu_index: 0,
            commands: command_palette::CommandPalette::default(),
            command_input,
            palette_open: false,
            palette_cursor: 0,
            palette_scroll: ScrollHandle::new(),
            swatch_scroll: ScrollHandle::new(),
            save_flow: save_flow::SaveFlow::default(),
            info: None,
            status: library_error
                .clone()
                .or(agent_error.clone())
                .unwrap_or_default()
                .into(),
            error: library_error.is_some() || agent_error.is_some(),
        }
    }
    pub(super) fn report<T>(
        &mut self,
        result: EditorResult<T>,
        message: impl Into<HintText>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(_) => {
                self.status = message.into();
                self.error = false;
            }
            Err(e) => {
                self.status = e.to_string().into();
                self.error = true;
            }
        }
        // Sampling, empty clicks and rejected edits do not invalidate the image.
        // Avoid another CPU render (and its input wait) for unchanged content.
        if !self.viewport.is_current(&self.editor) {
            self.request_render(cx);
        }
        cx.notify();
    }
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: declared boolean grid setting"
    )]
    pub(super) fn replace_editor(&mut self, mut editor: Editor, cx: &mut Context<Self>) {
        self.cancel_camera_animation();
        {
            welcome::record(&editor, cx);
            welcome::project_ready(cx);
        }
        self.pause_history(cx);
        self.timeline.dragging = false;
        self.timeline.expanded = false;
        self.zoom_reference_span = editor.camera.span;
        if let Some(session) = &mut self.agent.session
            && let Err(e) = session.replace(editor.project())
        {
            self.status = e.to_string().into();
            self.error = true;
        }
        self.agent.events.clear();
        self.agent.follow = false;
        self.agent.capture_generation = self.agent.capture_generation.wrapping_add(1);
        self.agent.capturing = false;
        if let Some(old) = self.agent.capture.take() {
            self.agent
                .retired
                .extend(old.views.into_iter().map(|v| v.2.image));
        }
        editor.comments_state.region = self.editor.comments_state.region;
        editor.comments_state.filter = self.editor.comments_state.filter;

        editor.retain_palette_library(&self.editor);
        self.editor = editor;
        self.editor
            .set_grid(self.preferences.grid)
            .expect("declared boolean grid setting");
        self.overlay_drag = None;
        self.overlay_hover = None;
        self.overlay_bounds.borrow_mut().clear();
        self.hover = None;
        self.drag = None;
        self.cursor = self.editor.display_snapshot().bounds.min;
        self.camera_view = Some(Op::Frame);
        self.camera_fitted = true;
        self.viewport.replace_editor(&mut self.editor);
        self.error = false;
        self.request_render(cx);
    }

    fn observe_window(window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.weak_entity();
        window.on_window_should_close(cx, move |_, cx| {
            weak.update(cx, |this, cx| {
                match this.save_flow.request_close(|| this.editor.needs_save()) {
                    save_flow::CloseRequest::Allow => true,
                    save_flow::CloseRequest::Blocked => false,
                    save_flow::CloseRequest::Confirm => {
                        cx.notify();
                        false
                    }
                }
            })
            .unwrap_or(true)
        });
        cx.observe_window_activation(window, |this, window, cx| {
            this.reduced_motion = quantum_app::reduce_motion::prefers_reduced_motion();
            if !window.is_window_active() {
                this.drag = None;
                this.hover = None;
            }
            cx.notify();
        })
        .detach();
    }

    #[expect(clippy::expect_used, reason = "internal invariant: constant keystroke")]
    fn command_input(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<crate::text_field::TextField> {
        let command_input = crate::text_field::TextField::design("Type a command…", cx);
        command_input.update(cx, |field, _| field.embed());
        cx.subscribe_in(
            &command_input,
            window,
            |this, field, event, window, cx| match event {
                quantum_ui::TextFieldEvent::Changed => {
                    this.commands.set_query(field.read(cx).text());
                    cx.notify();
                }
                quantum_ui::TextFieldEvent::Committed => {
                    this.command_palette_key(
                        &Keystroke::parse("enter").expect("constant keystroke"),
                        window,
                        cx,
                    );
                }
                _ => {}
            },
        )
        .detach();
        command_input
    }
}
