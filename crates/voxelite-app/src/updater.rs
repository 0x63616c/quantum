//! Optional Sparkle updates for the packaged macOS app.
//!
//! Menu actions can call [`check_for_updates`] on GPUI's main thread. Ordinary
//! `cargo run` builds have no bundled framework, so initialization is a no-op.
#![allow(unsafe_code)]

use libloading::os::unix::{Library, RTLD_LOCAL, RTLD_NOW};
use objc2::{
    MainThreadMarker, msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyClass, AnyObject},
};
use std::{cell::OnceCell, mem::ManuallyDrop};

thread_local! {
    // Only accessed on the main thread. Keep the controller alive for the app's
    // lifetime; Sparkle's asynchronous work can outlive an individual menu call.
    static CONTROLLER: OnceCell<Option<Retained<AnyObject>>> = const { OnceCell::new() };
}

/// Start Sparkle once on the main thread, if this is a bundle with the framework.
pub fn init() {
    if MainThreadMarker::new().is_none() {
        return;
    }
    CONTROLLER.with(|controller| {
        controller.get_or_init(load_controller);
    });
}

/// Whether the bundled updater is available on the main thread.
pub fn is_available() -> bool {
    if MainThreadMarker::new().is_none() {
        return false;
    }
    init();
    CONTROLLER.with(|slot| matches!(slot.get(), Some(Some(_))))
}

/// Handle the macOS “Check for Updates…” menu action on GPUI's main thread.
///
/// Does nothing if Sparkle is unavailable or an update check is already active.
pub fn check_for_updates() {
    if MainThreadMarker::new().is_none() {
        return;
    }
    init();
    CONTROLLER.with(|slot| {
        if let Some(Some(controller)) = slot.get() {
            // SAFETY: This retained object is an SPUStandardUpdaterController
            // created below, on the main thread. These signatures match the
            // pinned Sparkle headers; the returned updater is non-null and is
            // owned by the controller. nil is an allowed sender.
            unsafe {
                let updater: Retained<AnyObject> = msg_send![controller, updater];
                let can_check: bool = msg_send![&updater, canCheckForUpdates];
                if can_check {
                    let _: () = msg_send![controller, checkForUpdates: None::<&AnyObject>];
                }
            }
        }
    });
}

fn load_controller() -> Option<Retained<AnyObject>> {
    let executable = std::env::current_exe().ok()?;
    let macos = executable.parent()?;
    let contents = macos.parent()?;
    if macos.file_name()? != "MacOS"
        || contents.file_name()? != "Contents"
        || contents.parent()?.extension()? != "app"
    {
        return None;
    }
    let framework = contents.join("Frameworks/Sparkle.framework/Sparkle");
    if !framework.is_file() {
        return None;
    }

    // SAFETY: Only load the framework from this app's signed bundle, on the main
    // thread. The packager pins/verifies Sparkle and signs its nested code. Keep
    // the dlopen handle for the process lifetime: registered Objective-C classes
    // and asynchronous callbacks must never refer to an unloaded library.
    match unsafe { Library::open(Some(&framework), RTLD_NOW | RTLD_LOCAL) } {
        Ok(library) => {
            let _library = ManuallyDrop::new(library);
        }
        Err(error) => {
            eprintln!("Automatic updates unavailable: cannot load Sparkle ({error}).");
            return None;
        }
    }
    let Some(class) = AnyClass::get(c"SPUStandardUpdaterController") else {
        eprintln!("Automatic updates unavailable: Sparkle updater class is missing.");
        return None;
    };

    // SAFETY: The dynamically loaded class implements these selectors in the
    // pinned Sparkle release. objc2 tracks alloc/init ownership; both delegate
    // parameters accept nil, and the BOOL argument uses objc2's bool conversion.
    // load_controller is called only after checking MainThreadMarker above.
    unsafe {
        let allocated: Allocated<AnyObject> = msg_send![class, alloc];
        let controller: Option<Retained<AnyObject>> = msg_send![
            allocated,
            initWithStartingUpdater: true,
            updaterDelegate: None::<&AnyObject>,
            userDriverDelegate: None::<&AnyObject>,
        ];
        controller
    }
}
