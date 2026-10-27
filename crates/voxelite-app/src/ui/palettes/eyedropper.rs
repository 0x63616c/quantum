//! Main-thread AppKit sampler. The callback's borrowed color never leaves AppKit.
#![allow(unsafe_code)]
use super::*;
use block2::RcBlock;
use std::cell::Cell;
use futures_channel::oneshot;
use objc2::{MainThreadMarker, runtime::AnyClass};
use objc2_app_kit::{NSColor, NSColorSampler, NSColorSpace};

type Sample = Result<Option<[u8; 3]>, ()>;
fn available() -> bool {
    MainThreadMarker::new().is_some() && AnyClass::get(c"NSColorSampler").is_some()
}
async fn sample() -> Sample {
    if !available() {
        return Err(());
    }
    let sampler = NSColorSampler::new();
    let (sender, receiver) = oneshot::channel();
    let sender = Cell::new(Some(sender));
    let completion = RcBlock::new(move |color: *mut NSColor| {
        // SAFETY: AppKit invokes this block on the main thread with a valid,
        // borrowed NSColor for the duration of the callback, or nil on cancel.
        let result = unsafe { color.as_ref() }
            .map(|color| {
                let color = color
                    .colorUsingColorSpace(&NSColorSpace::sRGBColorSpace())
                    .ok_or(())?;
                let channels = [
                    color.redComponent(),
                    color.greenComponent(),
                    color.blueComponent(),
                ];
                if channels.iter().any(|v| !v.is_finite()) {
                    return Err(());
                }
                Ok(channels.map(|v| (v.clamp(0., 1.) * 255.).round() as u8))
            })
            .transpose();
        if let Some(sender) = sender.take() {
            let _ = sender.send(result);
        }
    });
    // SAFETY: The generated API takes a block with this exact signature.
    // AppKit retains the sampler for the session and copies the completion block.
    // We additionally keep both alive until the main-thread callback completes.
    unsafe {
        sampler.showSamplerWithSelectionHandler(&completion);
    }
    let result = receiver.await.map_err(|_| ())?;
    drop(completion);
    drop(sampler);
    result
}
impl EditorView {
    pub(super) fn palette_eyedropper(&self, cx: &Context<Self>) -> Stateful<Div> {
        let enabled = available() && !self.palettes.sampler_failed;
        Button::new("palette-eyedropper", "Pick from screen", self.theme)
            .icon("picker")
            .variant(ButtonVariant::Ghost)
            .size(voxelite_ui::tokens::ControlSize::Sm)
            .disabled(!enabled)
            .build()
            .when(enabled, |d| {
                d.on_click(cx.listener(|this, _, _, cx| {
                    if !this.flush_palette_fields(cx) {
                        return;
                    }
                    let Some(view) = this.editor.palette_view() else {
                        return;
                    };
                    let identity = (view.selected, view.state.slot);
                    this.palettes.before = selected_rgb(&view);
                    this.save_flow.begin_dialog();
                    // Defer AppKit until the EditorView/App borrow has ended.
                    cx.spawn(async move |this, cx| {
                        let result = sample().await;
                        let _ = this.update(cx, |this, cx| {
                            this.save_flow.finish_dialog();
                            match result {
                                Ok(Some(rgb))
                                    if this.editor.palette_view().is_some_and(|v| {
                                        (v.selected, v.state.slot) == identity
                                    }) =>
                                {
                                    this.manager_action(
                                        PaletteAction::SetColor { rgb, commit: true },
                                        cx,
                                    )
                                }
                                Err(()) => this.palettes.sampler_failed = true,
                                _ => {}
                            }
                            cx.notify();
                        });
                    })
                    .detach();
                }))
            })
    }
}
