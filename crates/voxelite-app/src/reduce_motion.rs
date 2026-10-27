//! Platform motion preferences without a UI dependency.

/// Whether camera transitions are enabled by both the user and accessibility settings.
pub fn camera_animation_enabled(pref: bool, reduced_motion: bool) -> bool {
    pref && !reduced_motion
}

/// Read macOS Reduce Motion, or return false on other platforms.
///
/// The NSWorkspace property is thread-safe to read, so no main-thread marker is
/// required. Missing NSWorkspace classes are treated as having no preference.
pub fn prefers_reduced_motion() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::prefers_reduced_motion()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[cfg(target_os = "macos")]
mod macos {
    #![allow(unsafe_code)]

    use objc2::{
        msg_send,
        rc::Retained,
        runtime::{AnyClass, AnyObject},
    };

    pub(super) fn prefers_reduced_motion() -> bool {
        let Some(class) = AnyClass::get(c"NSWorkspace") else {
            return false;
        };
        // SAFETY: NSWorkspace implements these AppKit selectors. sharedWorkspace
        // returns a non-null singleton retained by objc2; the property returns BOOL.
        // Reading this accessibility preference is thread-safe, including off-main.
        unsafe {
            let workspace: Retained<AnyObject> = msg_send![class, sharedWorkspace];
            msg_send![&workspace, accessibilityDisplayShouldReduceMotion]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_requires_preference_and_no_reduced_motion() {
        assert!(camera_animation_enabled(true, false));
        assert!(!camera_animation_enabled(false, false));
        assert!(!camera_animation_enabled(true, true));
        assert!(!camera_animation_enabled(false, true));
    }
}
