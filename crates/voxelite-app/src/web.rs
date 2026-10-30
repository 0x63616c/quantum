//! Browser startup and bounded file transfer; the editor itself is shared GPUI.
use crate::{
    actions, assets,
    editor::{Editor, EditorError, EditorResult},
    theme::Theme,
    ui,
};
use gpui::*;
use std::{borrow::Cow, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};

fn error(value: JsValue) -> EditorError {
    EditorError::Gesture(format!("Browser operation failed: {value:?}"))
}

pub(crate) fn store(key: &str, bytes: &[u8]) -> EditorResult<()> {
    let storage = web_sys::window()
        .unwrap()
        .local_storage()
        .map_err(error)?
        .ok_or_else(|| EditorError::Gesture("Browser storage is unavailable".into()))?;
    let text = std::str::from_utf8(bytes).map_err(|e| EditorError::Gesture(e.to_string()))?;
    storage.set_item(key, text).map_err(error)
}

pub(crate) fn load(key: &str) -> EditorResult<Option<Vec<u8>>> {
    let Some(storage) = web_sys::window().unwrap().local_storage().map_err(error)? else {
        return Ok(None);
    };
    Ok(storage
        .get_item(key)
        .map_err(error)?
        .map(String::into_bytes))
}

thread_local! {
    static UNSAVED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) fn set_unsaved(unsaved: bool) {
    UNSAVED.set(unsaved);
}

fn protect_unsaved_edits() -> Result<(), JsValue> {
    let callback = Closure::<dyn FnMut(web_sys::BeforeUnloadEvent)>::new(
        |event: web_sys::BeforeUnloadEvent| {
            if UNSAVED.get() {
                event.prevent_default();
                event.set_return_value("");
            }
        },
    );
    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("beforeunload", callback.as_ref().unchecked_ref())?;
    // One listener lives for the lifetime of this browser application.
    callback.forget();
    Ok(())
}

type Upload = Option<(String, Vec<u8>)>;

/// Start an upload during the user gesture, then read only a bounded selection.
pub(crate) fn upload(
    accept: &str,
    limit: usize,
) -> EditorResult<impl Future<Output = EditorResult<Upload>>> {
    let document = web_sys::window().unwrap().document().unwrap();
    let input: web_sys::HtmlInputElement = document
        .create_element("input")
        .map_err(error)?
        .unchecked_into();
    input.set_type("file");
    input.set_accept(accept);
    input.set_hidden(true);
    document
        .body()
        .unwrap()
        .append_child(&input)
        .map_err(error)?;
    let (sender, receiver) = futures_channel::oneshot::channel();
    let sender = Rc::new(std::cell::RefCell::new(Some(sender)));
    let selected_input = input.clone();
    let changed = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        if let Some(sender) = sender.borrow_mut().take() {
            let _ = sender.send(selected_input.files().and_then(|f| f.get(0)));
        }
    });
    input
        .add_event_listener_with_callback("change", changed.as_ref().unchecked_ref())
        .map_err(error)?;
    input
        .add_event_listener_with_callback("cancel", changed.as_ref().unchecked_ref())
        .map_err(error)?;
    input.click();
    Ok(async move {
        let selected = receiver.await;
        input.remove();
        drop(changed);
        let Some(file) = selected.map_err(|e| EditorError::Gesture(e.to_string()))? else {
            return Ok(None);
        };
        if file.size() > limit as f64 {
            return Err(EditorError::Gesture(
                "Selected file exceeds the size limit".into(),
            ));
        }
        let buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
            .await
            .map_err(error)?;
        Ok(Some((
            file.name(),
            js_sys::Uint8Array::new(&buffer).to_vec(),
        )))
    })
}

pub(crate) fn download(name: &str, mime: &str, bytes: &[u8]) -> EditorResult<()> {
    let parts = js_sys::Array::new();
    parts.push(&js_sys::Uint8Array::from(bytes));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime);
    let blob =
        web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).map_err(error)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(error)?;
    let document = web_sys::window().unwrap().document().unwrap();
    let link: web_sys::HtmlAnchorElement = document
        .create_element("a")
        .map_err(error)?
        .unchecked_into();
    link.set_href(&url);
    link.set_download(&name.replace(['/', '\\'], "-"));
    link.set_hidden(true);
    document
        .body()
        .unwrap()
        .append_child(&link)
        .map_err(error)?;
    link.click();
    link.remove();
    // Retain the object URL until the browser has consumed the navigation.
    let cleanup = Closure::once_into_js(move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    });
    web_sys::window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(cleanup.unchecked_ref(), 30_000)
        .map_err(error)?;
    Ok(())
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    gpui_web::init_logging();
    protect_unsaved_edits()?;
    let query = web_sys::window().unwrap().location().search()?;
    let backend = if query.contains("backend=webgl") {
        gpui_web::WebBackendPreference::WebGl
    } else if query.contains("backend=webgpu") {
        gpui_web::WebBackendPreference::WebGpu
    } else {
        gpui_web::WebBackendPreference::Auto
    };
    let platform = Rc::new(gpui_web::WebPlatform::new_with_backend(false, backend));
    let http = std::sync::Arc::new(platform.fetch_http_client());
    let editor = Editor::example().map_err(|e| JsValue::from_str(&e.to_string()))?;
    Application::with_platform(platform)
        .with_http_client(http)
        .with_assets(assets::Assets)
        .run(move |cx| {
            cx.text_system()
                .add_fonts(vec![
                    Cow::Borrowed(include_bytes!("../assets/fonts/Geist.ttf")),
                    Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf")),
                    Cow::Borrowed(include_bytes!("../assets/fonts/Geist-SemiBold.ttf")),
                    Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono.ttf")),
                ])
                .expect("bundled fonts");
            cx.set_text_rendering_mode(TextRenderingMode::Grayscale);
            actions::install(cx);
            let theme = if query.contains("theme=light") {
                Theme::Light
            } else {
                Theme::Dark
            };
            cx.open_window(
                WindowOptions {
                    titlebar: None,
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| ui::EditorView::new(editor, theme, None, window, cx)),
            )
            .expect("browser canvas");
            cx.activate(true);
        });
    Ok(())
}
