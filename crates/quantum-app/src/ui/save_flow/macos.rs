//! Main-thread AppKit save panel and its retained accessory action target.
#![allow(unsafe_code)]

use super::SaveFormat;
use anyhow::{Context as _, Result};
use block2::RcBlock;
use futures_channel::oneshot;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained, sel,
};
use objc2_app_kit::{
    NSApplication, NSModalResponseCancel, NSModalResponseOK, NSPopUpButton, NSSavePanel,
    NSTextField, NSView,
};
use objc2_foundation::{
    NSArray, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSURL,
};
use objc2_uniform_type_identifiers::UTType;
use std::{
    cell::Cell,
    path::{Path, PathBuf},
};

type Selection = Result<Option<(PathBuf, SaveFormat)>>;

struct FormatTargetIvars {
    panel: Retained<NSSavePanel>,
    content_types: Vec<Retained<UTType>>,
    formats: Vec<SaveFormat>,
    index: Cell<usize>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements. The target and its
    // AppKit ivars are confined to the main thread and have no custom Drop.
    #[unsafe(super = NSObject)]
    #[name = "QuantumSaveFormatTarget"]
    #[thread_kind = MainThreadOnly]
    #[ivars = FormatTargetIvars]
    struct FormatTarget;

    // SAFETY: NSObjectProtocol adds no requirements beyond NSObject.
    unsafe impl NSObjectProtocol for FormatTarget {}

    impl FormatTarget {
        // SAFETY: Only the NSPopUpButton configured below sends this action,
        // with itself as the sender, on the main thread.
        #[unsafe(method(formatChanged:))]
        fn format_changed(&self, sender: &NSPopUpButton) {
            if let Ok(index) = usize::try_from(sender.indexOfSelectedItem()) {
                self.select_format(index);
            }
        }
    }
);

impl FormatTarget {
    fn new(
        mtm: MainThreadMarker,
        panel: Retained<NSSavePanel>,
        formats: Vec<SaveFormat>,
    ) -> Result<Retained<Self>> {
        // Unknown extensions such as .voxl receive a dynamic UTType from macOS.
        let content_types = formats
            .iter()
            .map(|format| {
                UTType::typeWithFilenameExtension(&NSString::from_str(format.extension()))
                    .with_context(|| format!("No content type for {}", format.label()))
            })
            .collect::<Result<Vec<_>>>()?;
        let this = Self::alloc(mtm).set_ivars(FormatTargetIvars {
            panel,
            content_types,
            formats,
            index: Cell::new(0),
        });
        // SAFETY: Initialize the NSObject superclass after setting Rust ivars.
        Ok(unsafe { msg_send![super(this), init] })
    }

    fn select_format(&self, index: usize) {
        let Some(format) = self.ivars().formats.get(index) else {
            return;
        };
        let ivars = self.ivars();
        // Capture the user's current basename before AppKit changes the type.
        // Replacing the extension avoids names like "model.voxl.glb".
        let mut name = PathBuf::from(ivars.panel.nameFieldStringValue().to_string());
        name.set_extension(format.extension());
        ivars
            .panel
            .setAllowedContentTypes(&NSArray::from_slice(&[&*ivars.content_types[index]]));
        ivars
            .panel
            .setNameFieldStringValue(&NSString::from_str(&name.to_string_lossy()));
        ivars.index.set(index);
    }
}

pub(super) fn choose_save_path(
    directory: &Path,
    suggested: &str,
    formats: Vec<SaveFormat>,
    cx: &gpui::App,
) -> oneshot::Receiver<Selection> {
    let (sender, receiver) = oneshot::channel();
    let directory = directory.to_owned();
    let suggested = suggested.to_owned();
    // Defer AppKit until the EditorView/App borrow has ended. In particular,
    // runModal's nested event loop must not re-enter a borrowed GPUI context.
    cx.foreground_executor()
        .spawn(async move {
            let result = show_panel(&directory, &suggested, formats).await;
            let _ = sender.send(result);
        })
        .detach();
    receiver
}

async fn show_panel(directory: &Path, suggested: &str, formats: Vec<SaveFormat>) -> Selection {
    let mtm = MainThreadMarker::new().context("Save panel requires the main thread")?;
    let panel = NSSavePanel::savePanel(mtm);
    panel.setDirectoryURL(NSURL::from_directory_path(directory).as_deref());
    panel.setNameFieldStringValue(&NSString::from_str(suggested));
    panel.setCanCreateDirectories(true);
    panel.setExtensionHidden(false);
    panel.setCanSelectHiddenExtension(false);
    panel.setAllowsOtherFileTypes(false);

    let target = FormatTarget::new(mtm, panel.clone(), formats)?;
    let label = NSTextField::labelWithString(&NSString::from_str("File Format:"), mtm);
    label.sizeToFit();
    let popup =
        NSPopUpButton::initWithFrame_pullsDown(NSPopUpButton::alloc(mtm), NSRect::ZERO, false);
    popup.setAutoenablesItems(false);
    for format in &target.ivars().formats {
        popup.addItemWithTitle(&NSString::from_str(format.label()));
    }
    let initial_format = SaveFormat::from_extension(
        &target.ivars().formats,
        Path::new(suggested)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default(),
    );
    let initial_index = target
        .ivars()
        .formats
        .iter()
        .position(|format| *format == initial_format)
        .unwrap_or(0);
    popup.selectItemAtIndex(initial_index as isize);
    target.select_format(initial_index);
    popup.sizeToFit();

    // Use intrinsic control sizes so the accessory follows the OS's metrics.
    let label_size = label.frame().size;
    let popup_size = popup.frame().size;
    let height = label_size.height.max(popup_size.height) + 16.0;
    label.setFrameOrigin(NSPoint::new(0.0, (height - label_size.height) / 2.0));
    popup.setFrameOrigin(NSPoint::new(
        label_size.width + 12.0,
        (height - popup_size.height) / 2.0,
    ));
    let accessory = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(
            NSPoint::ZERO,
            NSSize::new(label_size.width + 12.0 + popup_size.width, height),
        ),
    );
    accessory.addSubview(&label);
    accessory.addSubview(&popup);
    panel.setAccessoryView(Some(&accessory));
    // SAFETY: This selector is defined above with NSPopUpButton's action ABI.
    // The local target stays retained across the sheet await / modal loop.
    unsafe {
        popup.setTarget(Some(&target));
        popup.setAction(Some(sel!(formatChanged:)));
    }

    let response = if let Some(window) = NSApplication::sharedApplication(mtm).keyWindow() {
        let (sender, receiver) = oneshot::channel();
        let sender = Cell::new(Some(sender));
        let completion = RcBlock::new(move |response| {
            if let Some(sender) = sender.take() {
                let _ = sender.send(response);
            }
        });
        panel.beginSheetModalForWindow_completionHandler(&window, &completion);
        receiver
            .await
            .context("Save panel closed without a response")
    } else {
        Ok(panel.runModal())
    };
    panel.orderOut(None);
    // SAFETY: Clear the unretained action target before releasing our owner.
    unsafe {
        popup.setTarget(None);
        popup.setAction(None);
    }
    let response = response?;
    if response == NSModalResponseCancel {
        return Ok(None);
    }
    anyhow::ensure!(
        response == NSModalResponseOK,
        "Save panel could not be shown"
    );
    let path = panel
        .URL()
        .and_then(|url| url.to_file_path())
        .context("Save panel did not return a file path")?;
    // Do not rewrite the confirmed path: AppKit already checked its extension
    // and obtained any necessary overwrite confirmation for exactly this URL.
    Ok(Some((
        path,
        target.ivars().formats[target.ivars().index.get()].clone(),
    )))
}
