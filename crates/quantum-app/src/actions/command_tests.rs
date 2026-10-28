use super::test_shell::assert_filtered_shell;
use super::*;
use gpui::{AssetSource, Keystroke};
use std::collections::HashSet;

#[test]
fn every_op_maps_to_exactly_one_host_command_and_back() {
    let host = quantum_app::app_host();
    let commands: Vec<_> = host.registry().commands().collect();
    assert_eq!(commands.len(), Op::ALL.len());
    let mut ids = HashSet::new();
    for op in Op::ALL {
        assert!(ids.insert(op.command_id()), "duplicate mapping: {op:?}");
        assert_eq!(Op::from_command_id(op.command_id()), Some(*op));
        assert_eq!(
            commands
                .iter()
                .filter(|(_, command)| command.id == op.command_id())
                .count(),
            1,
            "{op:?}"
        );
    }
    for (owner, command) in commands {
        let op = Op::from_command_id(&command.id).expect("command must have a dispatch Op");
        assert_eq!(op.command_id(), command.id);
        if op == Op::Grid {
            // Ownership moved; the persisted shell identity did not.
            assert_eq!(owner.as_str(), "dev.quantum.gizmo-grid");
        } else {
            assert!(command.id.starts_with(&format!("{owner}.")));
        }
    }
    assert_eq!(Op::from_command_id("dev.quantum.unknown.command"), None);
}

#[test]
fn disabling_plugins_removes_their_commands_from_every_shell_list() {
    let mut host = quantum_app::app_host();
    let plugins: Vec<_> = host
        .plugins()
        .filter(|(manifest, _)| manifest.can_disable)
        .map(|(manifest, _)| manifest.id.to_string())
        .collect();
    for plugin in plugins {
        let owned: Vec<_> = host
            .registry()
            .commands()
            .filter(|(owner, _)| owner.as_str() == plugin)
            .map(|(_, command)| Op::from_command_id(&command.id).unwrap())
            .collect();
        assert!(
            !owned.is_empty()
                || host
                    .registry()
                    .formats()
                    .any(|(owner, _)| owner.as_str() == plugin)
                || host
                    .registry()
                    .render_passes()
                    .any(|(owner, _)| owner.as_str() == plugin)
        );
        host.disable(&plugin).unwrap();
        assert!(host.registry().for_plugin(&plugin).is_none());
        for index in 0..MENUS.len() {
            assert!(
                menu_items(index, &host)
                    .iter()
                    .all(|op| !owned.contains(op))
            );
        }
        let shell = ShellCommands::from_host(&host);
        assert_filtered_shell(&shell, &owned);
        assert!(
            shell
                .menu_groups()
                .iter()
                .flat_map(|(_, ops)| ops)
                .all(|op| !owned.contains(op))
        );
        host.enable(&plugin).unwrap();
        assert_eq!(ShellCommands::from_host(&host).entries, SHELL.entries);
    }
}

#[test]
fn live_registry_disables_installed_mirror_actions_without_disabling_tools() {
    let mut host = quantum_app::app_host();
    let context = CommandContext::Editor(EditorState::default());
    for op in [Op::MirrorX, Op::MirrorY, Op::MirrorZ] {
        assert!(op.availability_in(&host, context).is_ok());
    }
    host.disable("dev.quantum.mirror").unwrap();
    for op in [Op::MirrorX, Op::MirrorY, Op::MirrorZ] {
        assert_eq!(
            op.availability_in(&host, context),
            Err("This command’s plugin is disabled")
        );
    }
    for op in [
        Op::Place,
        Op::Paint,
        Op::Erase,
        Op::Box,
        Op::Fill,
        Op::Picker,
        Op::New,
    ] {
        assert!(op.availability_in(&host, context).is_ok());
    }
    host.enable("dev.quantum.mirror").unwrap();
    for op in [Op::MirrorX, Op::MirrorY, Op::MirrorZ] {
        assert!(op.availability_in(&host, context).is_ok());
    }
}

#[test]
fn every_command_has_unique_metadata_and_appears_in_the_sheet() {
    let mut labels = HashSet::new();
    for op in Op::ALL {
        assert!(!op.label().is_empty(), "{op:?}");
        assert!(
            labels.insert(op.label().to_lowercase()),
            "duplicate label: {op:?}"
        );
        assert!(!op.icon().is_empty(), "{op:?}");
        assert!(
            crate::assets::Assets
                .load(&format!("icons/{}.svg", op.icon()))
                .unwrap()
                .is_some(),
            "icon is not bundled: {op:?}"
        );
    }
    for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
        let mut keys = HashSet::new();
        let sheet: Vec<_> = shortcut_sheet(platform).collect();
        for op in Op::ALL {
            assert_eq!(
                sheet.iter().filter(|entry| *entry == op).count(),
                usize::from(op.available_on(platform)),
                "{op:?}"
            );
            if let Some(binding) = op.binding(platform) {
                assert!(
                    keys.insert(binding.keystrokes().to_vec()),
                    "duplicate shortcut: {op:?}"
                );
            }
        }
    }
}

#[test]
fn sheet_includes_comment_and_all_mirror_shortcuts() {
    for (op, apple, other) in [
        (Op::Comment, "C", "C"),
        (Op::MirrorX, "⇧X", "Shift+X"),
        (Op::MirrorY, "⇧Y", "Shift+Y"),
        (Op::MirrorZ, "⇧Z", "Shift+Z"),
    ] {
        for (platform, expected) in [
            (ShortcutPlatform::Apple, apple),
            (ShortcutPlatform::WindowsLinux, other),
        ] {
            assert!(shortcut_sheet(platform).any(|entry| entry == op));
            assert_eq!(format_shortcut(op.key().unwrap(), platform), expected);
        }
    }
}

#[test]
fn guide_includes_every_available_bound_command_once() {
    for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
        let columns = keyboard_guide_sections(platform);
        let commands: Vec<_> = columns
            .iter()
            .flatten()
            .flat_map(|section| &section.rows)
            .filter_map(|(_, binding)| match binding {
                GuideShortcut::Action(op) => Some(*op),
                _ => None,
            })
            .collect();
        let expected: Vec<_> = shortcut_sheet(platform)
            .filter(|op| op.key().is_some())
            .collect();
        assert_eq!(commands.len(), expected.len(), "duplicate or missing rows");
        for op in expected {
            assert_eq!(
                commands.iter().filter(|entry| **entry == op).count(),
                1,
                "{op:?}"
            );
        }

        let tools = &columns[0][0];
        assert_eq!(tools.title, "Tools");
        for op in [Op::Comment, Op::MirrorX, Op::MirrorY, Op::MirrorZ] {
            assert!(tools.rows.iter().any(|(_, binding)| {
                matches!(binding, GuideShortcut::Action(entry) if *entry == op)
            }));
        }
    }
}

fn readme_shortcuts() -> String {
    use std::fmt::Write;
    let mut table = String::from("| Action | Apple | Windows / Linux |\n| --- | --- | --- |\n");
    for op in shortcut_sheet(ShortcutPlatform::Apple).filter(|op| op.key().is_some()) {
        let key = op.key().unwrap();
        let other = if op.available_on(ShortcutPlatform::WindowsLinux) {
            format_shortcut(key, ShortcutPlatform::WindowsLinux)
        } else {
            "—".into()
        };
        writeln!(
            table,
            "| {} | {} | {} |",
            op.label(),
            format_shortcut(key, ShortcutPlatform::Apple),
            other
        )
        .unwrap();
    }
    table
}

#[test]
fn readme_shortcuts_match_registry() {
    const START: &str = "<!-- BEGIN GENERATED COMMAND SHORTCUTS -->\n";
    const END: &str = "<!-- END GENERATED COMMAND SHORTCUTS -->";
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(
        readme.matches(START).count(),
        1,
        "missing or duplicate start marker"
    );
    assert_eq!(
        readme.matches(END).count(),
        1,
        "missing or duplicate end marker"
    );
    let (prefix, rest) = readme.split_once(START).unwrap();
    let (table, suffix) = rest.split_once(END).unwrap();
    let expected = readme_shortcuts();
    if std::env::var("UPDATE_README").as_deref() == Ok("1") {
        std::fs::write(path, format!("{prefix}{START}{expected}{END}{suffix}")).unwrap();
    } else {
        assert_eq!(
            table, expected,
            "README shortcuts drifted. Regenerate with: UPDATE_README=1 cargo test -p quantum-app --bin quantum readme_shortcuts_match_registry"
        );
    }
}

#[test]
fn editor_availability_preserves_live_preview_and_playback_rules() {
    let ready = EditorState {
        can_undo: true,
        can_redo: true,
        updates_available: true,
        ..EditorState::default()
    };
    let preview_blocked = [
        Op::Undo,
        Op::Redo,
        Op::Place,
        Op::Erase,
        Op::Paint,
        Op::Fill,
        Op::Box,
        Op::Picker,
        Op::MirrorX,
        Op::MirrorY,
        Op::MirrorZ,
        Op::AddLayer,
        Op::Comment,
        Op::ToolPicker,
    ];
    for op in Op::ALL {
        assert_eq!(
            op.availability(CommandContext::Editor(ready)),
            Ok(()),
            "{op:?}"
        );
        let preview = EditorState {
            preview: true,
            ..ready
        };
        assert_eq!(
            op.availability(CommandContext::Editor(preview)),
            if preview_blocked.contains(op) {
                Err("Return to live to edit")
            } else {
                Ok(())
            },
            "{op:?}"
        );
        let playing = EditorState {
            history_playing: true,
            can_undo: false,
            can_redo: false,
            ..preview
        };
        assert_eq!(
            op.availability(CommandContext::Editor(playing)).is_ok(),
            !preview_blocked.contains(op) || [Op::Undo, Op::Redo].contains(op),
            "{op:?}"
        );
        for state in [ready, preview, playing] {
            assert!(
                op.availability(CommandContext::Editor(EditorState {
                    dialog_open: true,
                    ..state
                }))
                .is_err(),
                "{op:?}"
            );
        }
    }
    let empty = CommandContext::Editor(EditorState::default());
    assert_eq!(Op::Undo.availability(empty), Err("Nothing to undo"));
    assert_eq!(Op::Redo.availability(empty), Err("Nothing to redo"));
    assert!(Op::CheckUpdates.availability(empty).is_err());
    let undo_only = CommandContext::Editor(EditorState {
        can_redo: false,
        ..ready
    });
    assert_eq!(Op::Undo.availability(undo_only), Ok(()));
    assert_eq!(Op::Redo.availability(undo_only), Err("Nothing to redo"));
    let redo_only = CommandContext::Editor(EditorState {
        can_undo: false,
        ..ready
    });
    assert_eq!(Op::Undo.availability(redo_only), Err("Nothing to undo"));
    assert_eq!(Op::Redo.availability(redo_only), Ok(()));
}

#[test]
fn disabled_reasons_identify_the_actual_blocker() {
    for preview in [false, true] {
        let state = EditorState {
            preview,
            ..EditorState::default()
        };
        assert_eq!(
            Op::CheckUpdates.availability(CommandContext::Editor(state)),
            Err("Updates are unavailable in this build.")
        );
        for op in Op::ALL {
            assert_eq!(
                op.availability(CommandContext::Editor(EditorState {
                    dialog_open: true,
                    ..state
                })),
                Err("File operation in progress"),
                "{op:?}"
            );
            assert_eq!(
                op.availability(CommandContext::Editor(EditorState {
                    pending_confirmation: true,
                    ..state
                })),
                Err("Finish saving or discarding changes first"),
                "{op:?}"
            );
        }
    }
}

#[test]
fn modal_availability_matches_dispatch_restrictions() {
    let state = CommandContext::Editor(EditorState {
        modal_open: true,
        can_undo: true,
        can_redo: true,
        updates_available: true,
        ..EditorState::default()
    });
    for op in Op::ALL {
        let expected = if [
            Op::Settings,
            Op::Quit,
            Op::Hide,
            Op::HideOthers,
            Op::ShowAll,
        ]
        .contains(op)
        {
            Ok(())
        } else {
            Err("Close the dialog to use this command")
        };
        assert_eq!(op.availability(state), expected, "{op:?}");
    }
}

#[test]
fn search_includes_every_supported_command_except_itself() {
    for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
        let search = search_on("", false, platform);
        for op in Op::ALL {
            assert_eq!(
                search.iter().filter(|entry| *entry == op).count(),
                usize::from(op.available_on(platform) && *op != Op::Commands),
                "{platform:?}: {op:?}"
            );
        }
    }
    assert_eq!(
        search_on("updates", false, ShortcutPlatform::Apple),
        vec![Op::CheckUpdates]
    );
    assert!(search_on("updates", false, ShortcutPlatform::WindowsLinux).is_empty());
}

#[test]
fn unavailable_updates_are_disabled_in_menus_with_a_short_reason() {
    let state = CommandContext::Editor(EditorState::default());
    let menus = SHELL.menus(true, false, |_| false, |op| op.availability(state).is_ok());
    let updates = menus.iter().flat_map(|menu| &menu.items).find(|item| matches!(item, MenuItem::Action { action, .. } if action.as_any().downcast_ref::<Op>() == Some(&Op::CheckUpdates)));
    assert!(matches!(
        updates,
        Some(MenuItem::Action { disabled: true, .. })
    ));
    assert_eq!(
        Op::CheckUpdates.availability(state),
        Err("Updates are unavailable in this build.")
    );
    assert!(search_on("updates", false, ShortcutPlatform::Apple).contains(&Op::CheckUpdates));
}

#[test]
fn search_preserves_tool_order_and_matches_labels_case_insensitively() {
    assert_eq!(
        search("", true),
        [
            Op::Place,
            Op::Erase,
            Op::Paint,
            Op::Fill,
            Op::Box,
            Op::Picker
        ]
    );
    assert_eq!(
        search("mIRroR", false),
        [Op::MirrorX, Op::MirrorY, Op::MirrorZ]
    );
    assert_eq!(search("comment", false), [Op::Comment]);
    assert!(search("comment", true).is_empty());
    assert!(!search("", false).contains(&Op::Commands));
}

#[test]
fn mac_menus_preserve_editor_commands_and_system_entries() {
    let menus = menus_for_platform(true, |op| op == Op::Grid, |op| op != Op::Undo);
    assert_eq!(
        menus
            .iter()
            .map(|menu| menu.name.as_ref())
            .collect::<Vec<_>>(),
        [
            "Quantum",
            "File",
            "Edit",
            "View",
            "Tools",
            "Layers",
            "Palette",
            "Agent",
            "Extensions",
            "Window",
            "Help"
        ]
    );
    assert!(menus[0].items.iter().any(|item| matches!(item, MenuItem::SystemMenu(menu) if menu.menu_type == gpui::SystemMenuType::Services)));
    let items: Vec<_> = menus.iter().flat_map(|menu| &menu.items).collect();
    for op in shortcut_sheet(ShortcutPlatform::Apple) {
        if matches!(op, Op::OpenRecent | Op::ClearRecents) {
            // Nested placement is asserted by welcome_and_recent_commands_share_the_registry.
            continue;
        }
        assert_eq!(items.iter().filter(|item| matches!(item, MenuItem::Action { action, .. } if action.as_any().downcast_ref::<Op>() == Some(&op))).count(), 1, "missing or duplicated {op:?}");
        let menu = menus
            .iter()
            .find(|menu| menu.name.as_ref() == op.menu(true))
            .unwrap();
        assert!(menu.items.iter().any(|item| matches!(item, MenuItem::Action { action, .. } if action.as_any().downcast_ref::<Op>() == Some(&op))), "wrong menu: {op:?}");
    }
    assert!(items.iter().any(|item| matches!(item, MenuItem::Action { action, disabled: true, os_action: Some(OsAction::Undo), .. } if action.as_any().downcast_ref::<Op>() == Some(&Op::Undo))));
    assert!(items.iter().any(|item| matches!(item, MenuItem::Action { action, checked: true, .. } if action.as_any().downcast_ref::<Op>() == Some(&Op::Grid))));
    for op in [
        ClipboardOp::Cut,
        ClipboardOp::Copy,
        ClipboardOp::Paste,
        ClipboardOp::SelectAll,
    ] {
        assert!(items.iter().any(|item| matches!(item, MenuItem::Action { action, os_action: Some(_), .. } if action.as_any().downcast_ref::<ClipboardOp>() == Some(&op))));
    }
}

#[test]
fn primary_actions_bind_command_on_apple_and_control_elsewhere() {
    for (op, apple, other) in [
        (Op::New, "cmd-n", "ctrl-n"),
        (Op::Open, "cmd-o", "ctrl-o"),
        (Op::Save, "cmd-s", "ctrl-s"),
        (Op::SaveAs, "cmd-shift-s", "ctrl-shift-s"),
        (Op::Quit, "cmd-q", "ctrl-q"),
        (Op::Undo, "cmd-z", "ctrl-z"),
        (Op::Redo, "cmd-shift-z", "ctrl-shift-z"),
        (Op::Commands, "cmd-k", "ctrl-k"),
        (Op::Settings, "cmd-,", "ctrl-,"),
    ] {
        for (platform, accepted, rejected) in [
            (ShortcutPlatform::Apple, apple, other),
            (ShortcutPlatform::WindowsLinux, other, apple),
        ] {
            let binding = op.binding(platform).unwrap();
            let accepted = Keystroke::parse(accepted).unwrap();
            let rejected = Keystroke::parse(rejected).unwrap();
            assert_eq!(binding.match_keystrokes(&[accepted]), Some(false), "{op:?}");
            assert_eq!(binding.match_keystrokes(&[rejected]), None, "{op:?}");
        }
    }
}

#[test]
fn tool_and_mirror_bindings_are_platform_independent() {
    for op in MENUS.iter().flat_map(|(_, items)| items) {
        let Some(key) = op
            .key()
            .filter(|key| !key.contains("secondary") && *op != Op::FullScreen)
        else {
            continue;
        };
        let stroke = Keystroke::parse(key).unwrap();
        for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
            assert_eq!(
                op.binding(platform)
                    .unwrap()
                    .match_keystrokes(std::slice::from_ref(&stroke)),
                Some(false),
                "{op:?}"
            );
        }
    }
}
