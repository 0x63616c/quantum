//! Frozen before the plugin metadata migration; keep changes to this baseline explicit.
use super::*;
use std::fmt::Write;

fn action_name(action: &dyn Action) -> String {
    if let Some(op) = action.as_any().downcast_ref::<Op>() {
        format!("{op:?}")
    } else {
        format!(
            "Clipboard::{:?}",
            action.as_any().downcast_ref::<ClipboardOp>().unwrap()
        )
    }
}

fn menu_snapshot(output: &mut String, menu: &Menu, platform: ShortcutPlatform, indent: &str) {
    writeln!(output, "{indent}menu {}", menu.name).unwrap();
    for item in &menu.items {
        match item {
            MenuItem::Separator => writeln!(output, "{indent}  ---").unwrap(),
            MenuItem::Submenu(menu) => {
                menu_snapshot(output, menu, platform, &format!("{indent}  "))
            }
            MenuItem::SystemMenu(menu) => {
                writeln!(output, "{indent}  system {}", menu.name).unwrap()
            }
            MenuItem::Action {
                name,
                action,
                os_action,
                checked,
                disabled,
            } => {
                let op = action.as_any().downcast_ref::<Op>();
                let icon = op.map(|op| op.icon()).unwrap_or("");
                let shortcut = key_bindings(platform)
                    .into_iter()
                    .find(|binding| binding.action().partial_eq(action.as_ref()))
                    .map(|binding| {
                        binding
                            .keystrokes()
                            .iter()
                            .map(|stroke| format_shortcut(&stroke.unparse(), platform))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .unwrap_or_default();
                let os_action = match os_action {
                    None => "",
                    Some(OsAction::Cut) => "Cut",
                    Some(OsAction::Copy) => "Copy",
                    Some(OsAction::Paste) => "Paste",
                    Some(OsAction::SelectAll) => "SelectAll",
                    Some(OsAction::Undo) => "Undo",
                    Some(OsAction::Redo) => "Redo",
                };
                writeln!(output, "{indent}  {} | {name} | icon={icon} | shortcut={shortcut} | os={os_action} | checked={checked} | disabled={disabled}", action_name(action.as_ref())).unwrap();
            }
        }
    }
}

fn shell_snapshot() -> String {
    let mut output = String::new();
    for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
        writeln!(output, "{platform:?}").unwrap();
        for menu in menus_for_platform(platform == ShortcutPlatform::Apple, |_| false, |_| true) {
            menu_snapshot(&mut output, &menu, platform, "");
        }
        writeln!(output, "in-window menus").unwrap();
        for (name, ops) in MENUS.iter() {
            writeln!(output, "  {name}: {ops:?}").unwrap();
        }
        writeln!(output, "Cmd-K").unwrap();
        for op in search_on("", false, platform) {
            writeln!(
                output,
                "  {op:?} | {} | {} | {}",
                op.label(),
                op.icon(),
                op.key()
                    .map(|key| format_shortcut(key, platform))
                    .unwrap_or_default()
            )
            .unwrap();
        }
        writeln!(output, "shortcut sheet").unwrap();
        for op in shortcut_sheet(platform) {
            writeln!(
                output,
                "  {op:?} | {} | {}",
                op.menu(platform == ShortcutPlatform::Apple),
                op.key()
                    .map(|key| format_shortcut(key, platform))
                    .unwrap_or_default()
            )
            .unwrap();
        }
        writeln!(output, "bindings").unwrap();
        for binding in key_bindings(platform) {
            let keys = binding
                .keystrokes()
                .iter()
                .map(|stroke| {
                    // The platform spells the primary modifier `cmd` on macOS and `super` elsewhere.
                    stroke
                        .unparse()
                        .split('-')
                        .map(|part| if part == "cmd" { "super" } else { part })
                        .collect::<Vec<_>>()
                        .join("-")
                })
                .collect::<Vec<_>>()
                .join(" ");
            let context = binding
                .predicate()
                .map(|predicate| predicate.to_string())
                .unwrap_or_else(|| "global".into());
            writeln!(
                output,
                "  {} | {keys} | {context}",
                action_name(binding.action())
            )
            .unwrap();
        }
    }
    output
}

#[test]
#[cfg(not(target_family = "wasm"))]
fn shell_matches_pre_plugin_snapshot() {
    let actual = shell_snapshot();
    if std::env::var("UPDATE_SHELL_SNAPSHOT").as_deref() == Ok("1") {
        std::fs::write(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/actions/shell.snapshot"),
            actual,
        )
        .unwrap();
    } else {
        assert_eq!(actual, include_str!("shell.snapshot"));
    }
}
