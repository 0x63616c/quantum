//! Explicit launch fixtures for UI review. Normal startup never enters this module.
use super::*;
use settings::Section;
use voxelite_plugin_api::CommentsAction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LaunchState {
    Menu(&'static str),
    Palette,
    History,
    HistoryPreview,
    Comments,
    CommentThread,
    CommentDraft,
    AgentActivity,
    ConnectAgent,
    About,
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
    Debug,
    Settings(Section),
}

impl LaunchState {
    pub(crate) const ALL: &[(&str, Self)] = &[
        ("chrome-menu-quantum", Self::Menu("Quantum")),
        ("chrome-menu-file", Self::Menu("File")),
        ("chrome-menu-edit", Self::Menu("Edit")),
        ("chrome-menu-view", Self::Menu("View")),
        ("chrome-menu-tools", Self::Menu("Tools")),
        ("chrome-menu-palette", Self::Menu("Palette")),
        ("chrome-menu-window", Self::Menu("Window")),
        ("chrome-menu-help", Self::Menu("Help")),
        ("panels-palette", Self::Palette),
        ("panels-history", Self::History),
        ("panels-history-preview", Self::HistoryPreview),
        ("panels-comments", Self::Comments),
        ("panels-comment-thread", Self::CommentThread),
        ("panels-comment-draft", Self::CommentDraft),
        ("panels-agent", Self::AgentActivity),
        ("popovers-connect-agent", Self::ConnectAgent),
        ("dialogs-about", Self::About),
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
                self.editor.save_version("Workshop study".into())?;
                self.panel = SidebarTab::History;
                if state == LaunchState::HistoryPreview {
                    self.editor.preview_history(None)?;
                }
                self.request_render(cx);
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
            LaunchState::AgentActivity => {
                // A rejected request is a real feed state with no invented document edits.
                self.agent
                    .events
                    .push_back(voxelite_mcp::attached::SessionEvent {
                        tool: "voxelite_apply_commands".into(),
                        label: "voxelite_apply_commands".into(),
                        success: false,
                        revision: self.editor.engine.revision(),
                        entry: None,
                        changes: Default::default(),
                        error: Some(
                            "Revision changed; inspect the document before retrying.".into(),
                        ),
                        inspection: None,
                        views: Vec::new(),
                    });
                self.agent.expanded = Some(0);
            }
            LaunchState::ConnectAgent => self.invoke(Op::Agent, window, cx),
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
            LaunchState::Debug => self.preferences.show_debug_overlay = true,
            LaunchState::Settings(section) => self.open_settings(section, window, cx),
        }
        cx.notify();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        for section in Section::ALL {
            assert!(
                LaunchState::ALL
                    .iter()
                    .any(|(_, state)| *state == LaunchState::Settings(section))
            );
        }
    }
}
