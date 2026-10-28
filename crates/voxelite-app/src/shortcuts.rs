//! Host-aware bindings and the shared shortcut hint formatter.
use gpui::{Keystroke, Modifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutPlatform {
    Apple,
    WindowsLinux,
}

impl ShortcutPlatform {
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Apple
        } else {
            Self::WindowsLinux
        }
    }

    // Resolve the contribution's `secondary` alias for the selected platform
    // before parsing or binding keys.
    pub fn binding_key(self, key: &str) -> String {
        // The native full-screen convention is Control-Command-F on Apple,
        // and F11 on Windows/Linux. Contributions retain the canonical binding.
        if key == "ctrl-cmd-f" && self == Self::WindowsLinux {
            return "f11".into();
        }
        key.split('-')
            .map(|part| {
                if part.eq_ignore_ascii_case("secondary") {
                    match self {
                        Self::Apple => "cmd",
                        Self::WindowsLinux => "ctrl",
                    }
                } else {
                    part
                }
            })
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Exact primary modifier for clipboard/select-all shortcuts. In particular,
    /// Control on Apple and AltGr (Control+Alt) must not trigger these commands.
    pub fn is_primary(self, modifiers: Modifiers) -> bool {
        modifiers
            == match self {
                Self::Apple => Modifiers::command(),
                Self::WindowsLinux => Modifiers::control(),
            }
    }
}

/// Bare navigation keys, plus Shift+Tab for reverse focus traversal. Modified
/// keys must not accidentally move the voxel cursor or apply the current tool.
pub fn navigation_key(stroke: &Keystroke) -> Option<&str> {
    (stroke.modifiers == Modifiers::none()
        || (stroke.key == "tab" && stroke.modifiers == Modifiers::shift()))
    .then_some(stroke.key.as_str())
}

/// Keys routed by the editor after its existing modifier policy is applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditorKey {
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Escape,
    Tab,
    Enter,
    Space,
    PreviousSwatch,
    NextSwatch,
    Other,
}

impl EditorKey {
    /// Decode a raw key without filtering modifiers (the focused palette uses this).
    pub(crate) fn from_key(key: &str) -> Self {
        const KEYS: &[(&str, EditorKey)] = &[
            ("left", EditorKey::Left),
            ("right", EditorKey::Right),
            ("up", EditorKey::Up),
            ("down", EditorKey::Down),
            ("pageup", EditorKey::PageUp),
            ("pagedown", EditorKey::PageDown),
            ("escape", EditorKey::Escape),
            ("tab", EditorKey::Tab),
            ("enter", EditorKey::Enter),
            ("space", EditorKey::Space),
            ("[", EditorKey::PreviousSwatch),
            ("]", EditorKey::NextSwatch),
        ];
        KEYS.iter()
            .find_map(|(name, key_kind)| (*name == key).then_some(*key_kind))
            .unwrap_or(Self::Other)
    }

    /// Apply the same bare-key / Shift+Tab policy as the other shell adapters.
    pub(crate) fn navigation(stroke: &Keystroke) -> Self {
        Self::from_key(navigation_key(stroke).unwrap_or_default())
    }
}

/// A semantic shortcut token. Symbols have both a vector asset and a text
/// equivalent, so rendering never needs to reverse-parse a formatted label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShortcutToken {
    Text(String),
    Symbol(ShortcutSymbol),
}

impl ShortcutToken {
    pub fn text(&self) -> &str {
        match self {
            Self::Text(text) => text,
            Self::Symbol(symbol) => symbol.text(),
        }
    }
}

macro_rules! shortcut_symbols {
    ($($variant:ident => ($text:literal, $name:literal)),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum ShortcutSymbol { $($variant),+ }

        impl ShortcutSymbol {
            pub const ALL: &[Self] = &[$(Self::$variant),+];

            pub fn text(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }

            pub fn path(self) -> &'static str {
                match self { $(Self::$variant => concat!("icons/shortcut-", $name, ".svg")),+ }
            }

            pub fn data(self) -> &'static [u8] {
                match self {
                    $(Self::$variant => include_bytes!(concat!("../assets/icons/shortcut-", $name, ".svg"))),+
                }
            }
        }
    };
}

shortcut_symbols! {
    Control => ("⌃", "control"),
    Option => ("⌥", "option"),
    Shift => ("⇧", "shift"),
    Command => ("⌘", "command"),
    Escape => ("⎋", "escape"),
    Return => ("↩", "return"),
    Tab => ("⇥", "tab"),
    Backspace => ("⌫", "backspace"),
    Delete => ("⌦", "delete"),
    PageUp => ("⇞", "page-up"),
    PageDown => ("⇟", "page-down"),
    Home => ("↖", "home"),
    End => ("↘", "end"),
    Space => ("␣", "space"),
    CapsLock => ("⇪", "caps-lock"),
    Up => ("↑", "up"),
    Down => ("↓", "down"),
    Left => ("←", "left"),
    Right => ("→", "right"),
}

/// Tokenize a GPUI binding without consulting the environment. The source
/// binding supplies both the actual modifiers and every user-facing hint.
pub fn shortcut_tokens(binding: &str, platform: ShortcutPlatform) -> Vec<ShortcutToken> {
    tokens(binding, platform, false)
}

/// Compact keycaps omit separators and use Shift/Return glyphs on every platform.
pub fn compact_shortcut_tokens(binding: &str, platform: ShortcutPlatform) -> Vec<ShortcutToken> {
    tokens(binding, platform, true)
}

#[expect(clippy::expect_used, reason = "internal invariant: valid shortcut")]
fn tokens(binding: &str, platform: ShortcutPlatform, compact: bool) -> Vec<ShortcutToken> {
    use ShortcutSymbol as S;
    use ShortcutToken::{Symbol, Text};

    let stroke = Keystroke::parse(&platform.binding_key(binding)).expect("valid shortcut");
    let modifiers = stroke.modifiers;
    let apple = platform == ShortcutPlatform::Apple;
    let mut parts = Vec::new();
    for (pressed, symbol, name) in [
        (modifiers.control, S::Control, "Ctrl"),
        (modifiers.alt, S::Option, "Alt"),
        (modifiers.shift, S::Shift, "Shift"),
        (modifiers.platform, S::Command, "Super"),
    ] {
        if pressed {
            parts.push(if apple || (compact && symbol == S::Shift) {
                Symbol(symbol)
            } else {
                Text(name.into())
            });
        }
    }
    if modifiers.function {
        parts.push(Text(if apple { "fn" } else { "Fn" }.into()));
    }
    let key = match stroke.key.as_str() {
        "escape" if apple => Symbol(S::Escape),
        "enter" if apple || compact => Symbol(S::Return),
        "tab" if apple => Symbol(S::Tab),
        // GPUI calls the Mac's Delete key `backspace`; `delete` is forward delete.
        "backspace" if apple => Symbol(S::Backspace),
        "delete" if apple => Symbol(S::Delete),
        "pageup" if apple => Symbol(S::PageUp),
        "pagedown" if apple => Symbol(S::PageDown),
        "home" if apple => Symbol(S::Home),
        "end" if apple => Symbol(S::End),
        "space" if apple => Symbol(S::Space),
        "capslock" if apple => Symbol(S::CapsLock),
        "up" => Symbol(S::Up),
        "down" => Symbol(S::Down),
        "left" => Symbol(S::Left),
        "right" => Symbol(S::Right),
        key => Text(match key {
            "escape" => "Esc".into(),
            "enter" => "Enter".into(),
            "tab" => "Tab".into(),
            "backspace" => "Backspace".into(),
            "delete" => "Delete".into(),
            "home" => "Home".into(),
            "end" => "End".into(),
            "space" => "Space".into(),
            "insert" => "Insert".into(),
            "capslock" => "Caps Lock".into(),
            "pageup" => "Page Up".into(),
            "pagedown" => "Page Down".into(),
            "click" => "Click".into(),
            "rightdrag" => "Right-drag".into(),
            key => key.to_uppercase(),
        }),
    };
    parts.push(key);
    if apple || compact {
        parts
    } else {
        let mut separated = Vec::new();
        for part in parts {
            if !separated.is_empty() {
                separated.push(Text("+".into()));
            }
            separated.push(part);
        }
        separated
    }
}

/// Plain string API for tests, accessibility and nonvisual clients.
pub fn format_shortcut(binding: &str, platform: ShortcutPlatform) -> String {
    shortcut_tokens(binding, platform)
        .iter()
        .map(ShortcutToken::text)
        .collect()
}

#[cfg(test)]
mod tests {

    #[test]
    fn compact_keycaps_keep_platform_modifiers_and_semantic_navigation_glyphs() {
        use super::{
            ShortcutPlatform, ShortcutSymbol as S, ShortcutToken as T, compact_shortcut_tokens,
        };
        for (platform, modifier) in [
            (ShortcutPlatform::Apple, T::Symbol(S::Command)),
            (ShortcutPlatform::WindowsLinux, T::Text("Ctrl".into())),
        ] {
            assert_eq!(
                compact_shortcut_tokens("secondary-shift-o", platform),
                match platform {
                    ShortcutPlatform::Apple =>
                        vec![T::Symbol(S::Shift), modifier, T::Text("O".into())],
                    ShortcutPlatform::WindowsLinux =>
                        vec![modifier, T::Symbol(S::Shift), T::Text("O".into())],
                }
            );
            assert_eq!(
                compact_shortcut_tokens("enter", platform),
                vec![T::Symbol(S::Return)]
            );
            assert_eq!(
                compact_shortcut_tokens("up", platform),
                vec![T::Symbol(S::Up)]
            );
        }
    }
    use super::*;

    #[test]
    fn formats_shortcuts_for_both_platforms() {
        for (binding, apple, other) in [
            ("secondary-n", "⌘N", "Ctrl+N"),
            ("secondary-o", "⌘O", "Ctrl+O"),
            ("secondary-s", "⌘S", "Ctrl+S"),
            ("secondary-shift-s", "⇧⌘S", "Ctrl+Shift+S"),
            ("secondary-alt-z", "⌥⌘Z", "Ctrl+Alt+Z"),
            ("shift-alt-secondary-s", "⌥⇧⌘S", "Ctrl+Alt+Shift+S"),
            ("shift-alt-ctrl-z", "⌃⌥⇧Z", "Ctrl+Alt+Shift+Z"),
            ("shift-x", "⇧X", "Shift+X"),
            ("f", "F", "F"),
            ("[", "[", "["),
            ("]", "]", "]"),
            ("SECONDARY-shift-s", "⇧⌘S", "Ctrl+Shift+S"),
            ("escape", "⎋", "Esc"),
            ("enter", "↩", "Enter"),
            ("tab", "⇥", "Tab"),
            ("shift-tab", "⇧⇥", "Shift+Tab"),
            ("backspace", "⌫", "Backspace"),
            ("secondary-backspace", "⌘⌫", "Ctrl+Backspace"),
            ("delete", "⌦", "Delete"),
            ("pageup", "⇞", "Page Up"),
            ("pagedown", "⇟", "Page Down"),
            ("home", "↖", "Home"),
            ("end", "↘", "End"),
            ("up", "↑", "↑"),
            ("down", "↓", "↓"),
            ("left", "←", "←"),
            ("right", "→", "→"),
            ("secondary-left", "⌘←", "Ctrl+←"),
            ("space", "␣", "Space"),
            ("capslock", "⇪", "Caps Lock"),
            ("f12", "F12", "F12"),
            ("secondary--", "⌘-", "Ctrl+-"),
            ("secondary-+", "⌘+", "Ctrl++"),
            ("alt-click", "⌥Click", "Alt+Click"),
            ("shift-rightdrag", "⇧Right-drag", "Shift+Right-drag"),
        ] {
            assert_eq!(format_shortcut(binding, ShortcutPlatform::Apple), apple);
            assert_eq!(
                format_shortcut(binding, ShortcutPlatform::WindowsLinux),
                other
            );
        }
    }

    #[test]
    fn apple_modifiers_always_follow_control_option_shift_command_order() {
        for binding in ["cmd-shift-alt-ctrl-s", "alt-ctrl-cmd-shift-s"] {
            assert_eq!(format_shortcut(binding, ShortcutPlatform::Apple), "⌃⌥⇧⌘S");
        }
    }

    #[test]
    fn tokens_preserve_symbols_and_text_without_parsing_labels() {
        use ShortcutSymbol::*;
        use ShortcutToken::{Symbol, Text};
        assert_eq!(
            shortcut_tokens("ctrl-alt-shift-cmd-s", ShortcutPlatform::Apple),
            vec![
                Symbol(Control),
                Symbol(Option),
                Symbol(Shift),
                Symbol(Command),
                Text("S".into())
            ]
        );
        assert_eq!(
            shortcut_tokens("secondary-left", ShortcutPlatform::WindowsLinux),
            vec![Text("Ctrl".into()), Text("+".into()), Symbol(Left)]
        );
        assert_eq!(
            shortcut_tokens("fn-secondary-+", ShortcutPlatform::Apple),
            vec![Symbol(Command), Text("fn".into()), Text("+".into())]
        );
    }

    #[test]
    fn every_shortcut_symbol_has_a_bundled_vector() {
        use gpui::AssetSource;
        for symbol in ShortcutSymbol::ALL {
            let data = crate::assets::Assets.load(symbol.path()).unwrap().unwrap();
            assert_eq!(data.as_ref(), symbol.data());
            let svg = std::str::from_utf8(&data).unwrap();
            assert!(svg.contains("<path"), "{symbol:?}");
            assert!(!svg.contains("<text"), "{symbol:?}");
        }
        for key in [
            "escape",
            "enter",
            "tab",
            "backspace",
            "delete",
            "pageup",
            "pagedown",
            "home",
            "end",
            "space",
            "capslock",
            "up",
            "down",
            "left",
            "right",
        ] {
            assert!(
                matches!(
                    shortcut_tokens(key, ShortcutPlatform::Apple).as_slice(),
                    [ShortcutToken::Symbol(_)]
                ),
                "{key}"
            );
        }
    }

    #[test]
    fn primary_shortcuts_require_the_exact_host_modifier() {
        for (binding, apple, other) in [
            ("cmd-a", true, false),
            ("ctrl-a", false, true),
            ("a", false, false),
            ("alt-a", false, false),
            ("ctrl-alt-a", false, false),
            ("cmd-alt-a", false, false),
            ("cmd-ctrl-a", false, false),
            ("cmd-shift-a", false, false),
            ("ctrl-shift-a", false, false),
            ("fn-cmd-a", false, false),
        ] {
            let modifiers = Keystroke::parse(binding).unwrap().modifiers;
            assert_eq!(
                ShortcutPlatform::Apple.is_primary(modifiers),
                apple,
                "{binding}"
            );
            assert_eq!(
                ShortcutPlatform::WindowsLinux.is_primary(modifiers),
                other,
                "{binding}"
            );
        }
    }

    #[test]
    fn editor_keys_preserve_raw_palette_and_modified_navigation_routing() {
        for (key, expected) in [
            ("left", EditorKey::Left),
            ("right", EditorKey::Right),
            ("up", EditorKey::Up),
            ("down", EditorKey::Down),
            ("pageup", EditorKey::PageUp),
            ("pagedown", EditorKey::PageDown),
            ("escape", EditorKey::Escape),
            ("tab", EditorKey::Tab),
            ("enter", EditorKey::Enter),
            ("space", EditorKey::Space),
            ("[", EditorKey::PreviousSwatch),
            ("]", EditorKey::NextSwatch),
            ("backspace", EditorKey::Other),
            ("a", EditorKey::Other),
        ] {
            let stroke = Keystroke::parse(key).unwrap();
            assert_eq!(EditorKey::navigation(&stroke), expected, "{key}");
            for modifier in ["cmd", "ctrl", "alt", "shift", "fn"] {
                let stroke = Keystroke::parse(&format!("{modifier}-{key}")).unwrap();
                assert_eq!(
                    EditorKey::from_key(&stroke.key),
                    expected,
                    "{modifier}-{key}"
                );
                let navigation =
                    if stroke.modifiers == Modifiers::shift() && expected == EditorKey::Tab {
                        EditorKey::Tab
                    } else {
                        EditorKey::Other
                    };
                assert_eq!(
                    EditorKey::navigation(&stroke),
                    navigation,
                    "{modifier}-{key}"
                );
            }
        }
        assert_eq!(EditorKey::from_key(""), EditorKey::Other);
        assert_eq!(EditorKey::from_key("unknown-key"), EditorKey::Other);
    }

    #[test]
    fn navigation_does_not_treat_modified_shortcuts_as_bare_keys() {
        for key in [
            "left",
            "right",
            "up",
            "down",
            "pageup",
            "pagedown",
            "enter",
            "escape",
            "backspace",
            "[",
            "]",
            "tab",
        ] {
            assert_eq!(navigation_key(&Keystroke::parse(key).unwrap()), Some(key));
            for modifier in ["cmd", "ctrl", "alt", "shift", "fn"] {
                let stroke = Keystroke::parse(&format!("{modifier}-{key}")).unwrap();
                let expected = (modifier == "shift" && key == "tab").then_some(key);
                assert_eq!(navigation_key(&stroke), expected, "{modifier}-{key}");
            }
        }
    }
}
