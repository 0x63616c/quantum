use super::*;

pub(super) fn assert_filtered_shell(shell: &ShellCommands, owned: &[Op]) {
    for platform in [ShortcutPlatform::Apple, ShortcutPlatform::WindowsLinux] {
        assert_eq!(
            shell.shortcut_sheet(platform).collect::<Vec<_>>(),
            SHELL
                .shortcut_sheet(platform)
                .filter(|op| !owned.contains(op))
                .collect::<Vec<_>>()
        );
        for tools_only in [false, true] {
            assert_eq!(
                shell.search("", tools_only, platform),
                SHELL
                    .search("", tools_only, platform)
                    .into_iter()
                    .filter(|op| !owned.contains(op))
                    .collect::<Vec<_>>()
            );
        }
        let expected = SHELL
            .key_bindings(platform)
            .into_iter()
            .filter(|binding| {
                binding
                    .action()
                    .as_any()
                    .downcast_ref::<Op>()
                    .is_none_or(|op| !owned.contains(op))
            })
            .collect::<Vec<_>>();
        let actual = shell.key_bindings(platform);
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(&expected) {
            assert!(actual.action().partial_eq(expected.action()));
            assert_eq!(actual.keystrokes(), expected.keystrokes());
            assert_eq!(actual.predicate(), expected.predicate());
        }
        let menus = shell.menus(
            platform == ShortcutPlatform::Apple,
            false,
            |_| false,
            |_| true,
        );
        let menu_ops: Vec<_> = menus
            .iter()
            .flat_map(|menu| &menu.items)
            .flat_map(|item| match item {
                MenuItem::Action { action, .. } => action
                    .as_any()
                    .downcast_ref::<Op>()
                    .copied()
                    .into_iter()
                    .collect::<Vec<_>>(),
                MenuItem::Submenu(menu) if menu.name == Op::OpenRecent.label() => {
                    let mut ops = vec![Op::OpenRecent];
                    ops.extend(menu.items.iter().filter_map(|item| match item {
                        MenuItem::Action { action, .. } => {
                            action.as_any().downcast_ref::<Op>().copied()
                        }
                        _ => None,
                    }));
                    ops
                }
                _ => Vec::new(),
            })
            .collect();
        assert!(owned.iter().all(|op| !menu_ops.contains(op)));
        for op in shell.shortcut_sheet(platform) {
            assert_eq!(
                menu_ops.iter().filter(|entry| **entry == op).count(),
                1,
                "{op:?}"
            );
        }
        assert!(menus.iter().all(|menu| !menu.items.is_empty()));
    }
}
