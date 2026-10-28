//! Explicit launch fixtures for UI review. Normal startup never enters this module.
use super::*;
use quantum_plugin_api::CommentsAction;
use settings::Section;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LaunchState {
    Menu(&'static str),
    Palette,
    History,
    HistoryPreview,
    CommentsEmpty,
    Comments,
    CommentThread,
    CommentDraft,
    AgentActivity,
    AgentInspection,
    ConnectAgent,
    About,
    Feedback,
    CrashReport,
    Acknowledgements,
    KeyboardGuide,
    Commands,
    Tools,
    Extensions,
    UnsavedChanges,
    SaveWaiting,
    PaletteChooser,
    PaletteManager,
    PaletteMenu,
    PaletteDelete,
    Debug,
    Settings(Section),
}

impl LaunchState {
    pub(crate) const ALL: &[(&str, Self)] = &[
        ("chrome-menu-file", Self::Menu("File")),
        ("chrome-menu-edit", Self::Menu("Edit")),
        ("chrome-menu-view", Self::Menu("View")),
        ("chrome-menu-tools", Self::Menu("Tools")),
        ("chrome-menu-palette", Self::Menu("Palette")),
        ("chrome-menu-layers", Self::Menu("Layers")),
        ("chrome-menu-agent", Self::Menu("Agent")),
        ("chrome-menu-extensions", Self::Menu("Extensions")),
        ("chrome-menu-window", Self::Menu("Window")),
        ("chrome-menu-help", Self::Menu("Help")),
        ("panels-palette", Self::Palette),
        ("panels-history", Self::History),
        ("panels-history-preview", Self::HistoryPreview),
        ("panels-comments-empty", Self::CommentsEmpty),
        ("panels-comments", Self::Comments),
        ("panels-comment-thread", Self::CommentThread),
        ("panels-comment-draft", Self::CommentDraft),
        ("panels-agent", Self::AgentActivity),
        ("panels-agent-inspection", Self::AgentInspection),
        ("popovers-connect-agent", Self::ConnectAgent),
        ("dialogs-about", Self::About),
        ("dialogs-feedback", Self::Feedback),
        ("dialogs-crash-report", Self::CrashReport),
        ("dialogs-acknowledgements", Self::Acknowledgements),
        ("dialogs-keyboard-guide", Self::KeyboardGuide),
        ("dialogs-commands", Self::Commands),
        ("dialogs-tools", Self::Tools),
        ("dialogs-extensions", Self::Extensions),
        ("dialogs-unsaved-changes", Self::UnsavedChanges),
        ("dialogs-save-waiting", Self::SaveWaiting),
        ("popovers-palette-chooser", Self::PaletteChooser),
        ("dialogs-palettes", Self::PaletteManager),
        ("popovers-palette-menu", Self::PaletteMenu),
        ("dialogs-palette-delete", Self::PaletteDelete),
        ("overlays-debug", Self::Debug),
        ("settings-general", Self::Settings(Section::General)),
        ("settings-appearance", Self::Settings(Section::Appearance)),
        ("settings-viewport", Self::Settings(Section::Viewport)),
        ("settings-shortcuts", Self::Settings(Section::Shortcuts)),
        ("settings-agent", Self::Settings(Section::Agent)),
        ("settings-updates", Self::Settings(Section::Updates)),
        ("settings-advanced", Self::Settings(Section::Advanced)),
    ];

    pub(crate) fn parse(name: &str) -> anyhow::Result<Self> {
        Self::ALL
            .iter()
            .find_map(|(key, state)| (*key == name).then_some(*state))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown --show state {name:?}; expected one of: {}",
                    Self::ALL
                        .iter()
                        .map(|(key, _)| *key)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }
}

impl EditorView {
    pub(super) fn show_capture(
        &mut self,
        state: LaunchState,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        self.capture_state = Some(state);
        match state {
            LaunchState::Menu(name) => {
                self.menu = Some(
                    MENUS
                        .iter()
                        .position(|(label, _)| *label == name)
                        .ok_or_else(|| {
                            anyhow::anyhow!("capture menu {name:?} is not registered")
                        })?,
                );
                self.menu_index = 0;
                window.focus(&self.modal_focus, cx);
            }
            LaunchState::Palette => self.invoke(Op::ShowPalette, window, cx),
            LaunchState::History | LaunchState::HistoryPreview => {
                self.editor.add_layer()?;
                let version = self.editor.save_version("Workshop study".into())?;
                self.panel = SidebarTab::History;
                if state == LaunchState::HistoryPreview {
                    // Preserve the model and window title while showing read-only controls.
                    self.editor.preview_version(version)?;
                }
                self.request_render(cx);
            }
            LaunchState::CommentsEmpty => {
                // The example has an open review note but no resolved threads.
                self.editor.comments_action(CommentsAction::Filter(1))?;
                self.panel = SidebarTab::Comments;
            }
            LaunchState::Comments | LaunchState::CommentThread | LaunchState::CommentDraft => {
                self.panel = SidebarTab::Comments;
                self.editor
                    .comments_action(CommentsAction::Anchor([0, 0, 0]))?;
                if state != LaunchState::CommentDraft {
                    self.editor.comments_action(CommentsAction::Post(
                        "Try a warmer highlight on the roof.".into(),
                    ))?;
                    if state == LaunchState::Comments {
                        self.editor.comments_action(CommentsAction::Select(None))?;
                    }
                }
                self.request_render(cx);
            }
            LaunchState::AgentActivity | LaunchState::AgentInspection => {
                self.capture_agent_activity(state, cx)?;
            }
            LaunchState::ConnectAgent => self.invoke(Op::Agent, window, cx),
            LaunchState::Feedback | LaunchState::CrashReport => {
                feedback::fixture(state == LaunchState::CrashReport, cx);
            }
            LaunchState::About => self.invoke(Op::About, window, cx),
            LaunchState::Acknowledgements => {
                self.info = Some(("Acknowledgements", InfoContent::Acknowledgements));
                window.focus(&self.modal_focus, cx);
            }
            LaunchState::KeyboardGuide => self.invoke(Op::Help, window, cx),
            LaunchState::Commands => self.invoke(Op::Commands, window, cx),
            LaunchState::Tools => self.invoke(Op::ToolPicker, window, cx),
            LaunchState::Extensions => self.invoke(Op::Extensions, window, cx),
            LaunchState::UnsavedChanges => {
                self.editor.add_layer()?;
                self.invoke(Op::New, window, cx);
            }
            LaunchState::SaveWaiting => {
                // Hold the app-owned waiting surface without launching a desktop portal.
                self.save_flow.picker.begin();
                self.save_flow.begin_dialog();
                self.status = "Waiting for file dialog…".into();
            }
            LaunchState::PaletteChooser => self.invoke(Op::Palette, window, cx),
            LaunchState::PaletteManager | LaunchState::PaletteMenu => {
                self.invoke(Op::ManagePalettes, window, cx);
                if state == LaunchState::PaletteMenu {
                    self.capture_palette_menu();
                }
            }
            LaunchState::PaletteDelete => {
                // Keep the copied library in memory; opening a capture must not write preferences.
                self.editor.palette_action_with(
                    PaletteAction::Edit {
                        operation: quantum_plugin_api::PaletteOperation::Duplicate,
                        name: "Workshop colors".into(),
                        color: String::new(),
                    },
                    |_| Ok(()),
                )?;
                self.invoke(Op::ManagePalettes, window, cx);
                self.capture_palette_delete();
            }
            LaunchState::Debug => {
                // Appearance notifications resync from this global. Keep the override
                // in memory so they cannot dismiss the fixture or persist a preference.
                cx.global_mut::<settings::SettingsState>()
                    .values
                    .show_debug_overlay = true;
                self.sync_preferences(window, cx);
            }
            LaunchState::Settings(section) => self.open_settings(section, window, cx),
        }
        cx.notify();
        Ok(())
    }
    fn capture_agent_activity(
        &mut self,
        state: LaunchState,
        cx: &mut Context<Self>,
    ) -> anyhow::Result<()> {
        // Reuse a real committed example entry so the expanded Revert control
        // points to an actual transaction. The second event exercises failure UI.
        let entry = self
            .editor
            .engine
            .history()
            .find(|entry| entry.label == "Create named version")
            .ok_or_else(|| anyhow::anyhow!("agent capture requires the example version entry"))?;
        self.agent
            .events
            .push_back(quantum_mcp::attached::SessionEvent {
                tool: "quantum_versions_save".into(),
                label: entry.label.clone(),
                success: true,
                revision: entry.result_revision,
                entry: Some(entry.id),
                changes: entry.changes.clone(),
                error: None,
                inspection: None,
                views: Vec::new(),
            });
        self.agent
            .events
            .push_back(quantum_mcp::attached::SessionEvent {
                tool: "quantum_voxels_batch".into(),
                label: "quantum_voxels_batch".into(),
                success: false,
                revision: self.editor.engine.revision(),
                entry: None,
                changes: Default::default(),
                error: Some("Revision changed; inspect the document before retrying.".into()),
                inspection: None,
                views: Vec::new(),
            });
        // The newest event is painted first. Keep its expanded controls visible.
        self.agent.events.swap(0, 1);
        self.agent.expanded = Some(1);
        if state == LaunchState::AgentInspection {
            self.capture_agent_views(
                Arc::new(self.editor.snapshot().clone()),
                ["iso", "front", "right"].map(str::to_owned).into(),
                self.editor.engine.revision(),
                cx,
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{LaunchState, MENUS, Section};

    #[test]
    fn empty_comments_fixture_has_no_resolved_threads() {
        let mut editor = super::Editor::example().unwrap();
        editor
            .comments_action(super::CommentsAction::Filter(1))
            .unwrap();
        assert!(editor.comments_view().unwrap().comments.is_empty());
        assert!(!editor.engine.is_dirty());
    }

    #[test]
    fn capture_states_cover_registered_menus_and_settings() {
        for (menu, _) in MENUS.iter() {
            assert!(
                LaunchState::ALL
                    .iter()
                    .any(|(_, state)| matches!(state, LaunchState::Menu(name) if name == menu)),
                "missing {menu} menu capture"
            );
        }
        let mut names = std::collections::BTreeSet::new();
        for (name, state) in LaunchState::ALL {
            assert!(names.insert(name), "duplicate state {name}");
            if let LaunchState::Menu(menu) = state {
                assert!(
                    MENUS.iter().any(|(name, _)| name == menu),
                    "unregistered menu {menu}"
                );
            }
        }
        for section in Section::ALL {
            assert!(
                LaunchState::ALL
                    .iter()
                    .any(|(_, state)| *state == LaunchState::Settings(section))
            );
        }
    }
}
