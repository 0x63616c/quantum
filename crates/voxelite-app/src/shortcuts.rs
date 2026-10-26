//! Host-aware bindings and the shared shortcut hint formatter.
use gpui::{Keystroke, Modifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShortcutPlatform {
    Apple,
    WindowsLinux,
}

impl ShortcutPlatform {
    pub fn current() -> Self {
        #[cfg(target_family = "wasm")]
        {
            thread_local! {
                static PLATFORM: ShortcutPlatform = browser_platform();
            }
            PLATFORM.with(|platform| *platform)
        }
        #[cfg(not(target_family = "wasm"))]
        {
            if cfg!(target_os = "macos") {
                Self::Apple
            } else {
                Self::WindowsLinux
            }
        }
    }

    // GPUI's `secondary` alias uses target_os, which cannot identify the host
    // of a wasm build. Resolve it explicitly before parsing or binding keys.
    pub fn binding_key(self, key: &str) -> String {
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

    /// Option produces text on Apple; AltGr produces text on Windows/Linux.
    pub fn allows_text(self, modifiers: Modifiers) -> bool {
        if modifiers.platform || modifiers.function {
            return false;
        }
        match self {
            Self::Apple => !modifiers.control,
            Self::WindowsLinux => modifiers.control == modifiers.alt,
        }
    }

    #[cfg(any(test, target_family = "wasm"))]
    fn from_browser(platform_data: &str, platform: &str, user_agent: &str) -> Self {
        // Prefer explicit platform information over the legacy UA, which may
        // contain compatibility tokens or a spoofed OS. Fall back when absent
        // or unrecognized (Safari/Firefox do not provide userAgentData).
        for value in [platform_data, platform] {
            let value = value.trim().to_ascii_lowercase();
            if value.starts_with("mac")
                || matches!(
                    value.as_str(),
                    "iphone" | "ipad" | "ipod" | "ios" | "ipados"
                )
            {
                return Self::Apple;
            }
            if value.starts_with("win")
                || value.starts_with("linux")
                || matches!(value.as_str(), "android" | "chrome os" | "cros")
            {
                return Self::WindowsLinux;
            }
        }
        if user_agent
            .split(|c: char| !c.is_ascii_alphabetic())
            .any(|token| {
                ["mac", "macintosh", "iphone", "ipad", "ipod"]
                    .iter()
                    .any(|apple| token.eq_ignore_ascii_case(apple))
            })
        {
            Self::Apple
        } else {
            Self::WindowsLinux
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

#[cfg(target_family = "wasm")]
fn browser_platform() -> ShortcutPlatform {
    use wasm_bindgen::JsValue;

    // userAgentData is not available in all browsers. Reflect also handles
    // missing navigator properties without requiring unstable web-sys APIs.
    let property = |object: &JsValue, name: &str| {
        js_sys::Reflect::get(object, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED)
    };
    let navigator = property(&js_sys::global(), "navigator");
    let data = property(&navigator, "userAgentData");
    ShortcutPlatform::from_browser(
        &property(&data, "platform").as_string().unwrap_or_default(),
        &property(&navigator, "platform")
            .as_string()
            .unwrap_or_default(),
        &property(&navigator, "userAgent")
            .as_string()
            .unwrap_or_default(),
    )
}

/// Format a GPUI binding without consulting the environment. The same source
/// binding supplies both the actual modifiers and every user-facing hint.
pub fn format_shortcut(binding: &str, platform: ShortcutPlatform) -> String {
    let stroke = Keystroke::parse(&platform.binding_key(binding)).expect("valid shortcut");
    let modifiers = stroke.modifiers;
    let apple = platform == ShortcutPlatform::Apple;
    let mut parts = Vec::new();
    for (pressed, symbol, name) in [
        (modifiers.control, "⌃", "Ctrl"),
        (modifiers.alt, "⌥", "Alt"),
        (modifiers.shift, "⇧", "Shift"),
        (modifiers.platform, "⌘", "Super"),
        (modifiers.function, "fn", "Fn"),
    ] {
        if pressed {
            parts.push(if apple { symbol } else { name });
        }
    }
    let key = match stroke.key.as_str() {
        "escape" if apple => "⎋".into(),
        "enter" if apple => "↩".into(),
        "tab" if apple => "⇥".into(),
        // GPUI calls the Mac's Delete key `backspace`; `delete` is forward delete.
        "backspace" if apple => "⌫".into(),
        "delete" if apple => "⌦".into(),
        "pageup" if apple => "⇞".into(),
        "pagedown" if apple => "⇟".into(),
        "home" if apple => "↖".into(),
        "end" if apple => "↘".into(),
        "space" if apple => "␣".into(),
        "capslock" if apple => "⇪".into(),
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
        "up" => "↑".into(),
        "down" => "↓".into(),
        "left" => "←".into(),
        "right" => "→".into(),
        "pageup" => "Page Up".into(),
        "pagedown" => "Page Down".into(),
        "click" => "Click".into(),
        "rightdrag" => "Right-drag".into(),
        key => key.to_uppercase(),
    };
    parts.push(&key);
    parts.join(if apple { "" } else { "+" })
}

#[cfg(test)]
mod tests {
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
    fn bundled_fallback_renders_every_apple_shortcut_symbol() {
        let fonts = crate::assets::fonts();
        let family = crate::assets::font("Geist").fallbacks.unwrap();
        let fallback = fonts
            .iter()
            .find_map(|bytes| {
                let face = ttf_parser::Face::parse(bytes, 0).unwrap();
                face.names()
                    .into_iter()
                    .any(|name| {
                        name.name_id == ttf_parser::name_id::FAMILY
                            && name.to_string().as_deref() == Some(&family.fallback_list()[0])
                    })
                    .then_some(face)
            })
            .expect("registered shortcut fallback family");
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
            let hint = format_shortcut(
                &format!("ctrl-alt-shift-cmd-{key}"),
                ShortcutPlatform::Apple,
            );
            for symbol in hint.chars() {
                assert!(
                    fallback.glyph_index(symbol).is_some_and(|id| id.0 != 0),
                    "missing {symbol}"
                );
            }
        }
    }

    #[test]
    fn detects_apple_browser_platforms_and_fallbacks() {
        for (data, platform, agent) in [
            ("macOS", "", ""),
            ("", "MacIntel", ""),
            ("", "", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"),
            ("", "iPhone", ""),
            ("", "iPad", ""),
            (
                "",
                "",
                "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X)",
            ),
            ("", "", "Mozilla/5.0 (iPad; CPU OS 18_0 like Mac OS X)"),
            ("MACOS", "", ""),
            ("iOS", "", ""),
            ("iPadOS", "", ""),
            ("", "iPod", ""),
            (
                "",
                "",
                "Mozilla/5.0 (iPod touch; CPU iPhone OS 18_0 like Mac OS X)",
            ),
            ("macOS", "Win32", "Mozilla/5.0 (Windows NT 10.0)"),
            ("Unknown", "MacIntel", ""),
        ] {
            assert_eq!(
                ShortcutPlatform::from_browser(data, platform, agent),
                ShortcutPlatform::Apple
            );
        }
        for (data, platform, agent) in [
            (
                "Windows",
                "Win32",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
            ),
            ("Linux", "Linux x86_64", "Mozilla/5.0 (X11; Linux x86_64)"),
            ("Android", "Linux armv8l", "Mozilla/5.0 (Linux; Android 15)"),
            ("", "", ""),
            (
                "Windows",
                "MacIntel",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X)",
            ),
            (
                "",
                "Linux x86_64",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X)",
            ),
            ("", "", "Emacs/30.1"),
            ("Chrome OS", "", ""),
        ] {
            assert_eq!(
                ShortcutPlatform::from_browser(data, platform, agent),
                ShortcutPlatform::WindowsLinux
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
    fn text_entry_preserves_option_and_altgr_without_inserting_commands() {
        for (binding, apple, other) in [
            ("e", true, true),
            ("shift-e", true, true),
            ("alt-e", true, false),
            ("alt-shift-e", true, false),
            ("ctrl-alt-e", false, true),
            ("ctrl-alt-shift-e", false, true),
            ("cmd-e", false, false),
            ("ctrl-e", false, false),
            ("cmd-ctrl-alt-e", false, false),
            ("fn-e", false, false),
        ] {
            let modifiers = Keystroke::parse(binding).unwrap().modifiers;
            assert_eq!(
                ShortcutPlatform::Apple.allows_text(modifiers),
                apple,
                "{binding}"
            );
            assert_eq!(
                ShortcutPlatform::WindowsLinux.allows_text(modifiers),
                other,
                "{binding}"
            );
        }
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
