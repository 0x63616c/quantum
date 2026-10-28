//! Ordered background settings writes, drained on graceful shutdown.
use super::*;

fn path() -> std::path::PathBuf {
    voxelite_app::local_settings::data_dir().join("settings.json")
}
pub(in crate::ui) fn init(override_theme: Option<Theme>, cx: &mut App) {
    if cx.has_global::<SettingsState>() {
        return;
    }
    init_loaded(override_theme, Preferences::load(&path()), cx);
}

pub(in crate::ui) fn init_loaded(
    override_theme: Option<Theme>,
    loaded: voxelite_app::editor::EditorResult<Preferences>,
    cx: &mut App,
) {
    init_at(override_theme, loaded, path(), cx);
}

pub(in crate::ui) fn init_at(
    override_theme: Option<Theme>,
    loaded: voxelite_app::editor::EditorResult<Preferences>,
    destination: std::path::PathBuf,
    cx: &mut App,
) {
    let (mut values, error) = match loaded {
        Ok(values) => (values, None),
        Err(error) => (
            Preferences::default(),
            Some(format!("Could not load settings: {error}")),
        ),
    };
    if let Some(theme) = override_theme {
        values.appearance = if theme == Theme::Light {
            Appearance::Light
        } else {
            Appearance::Dark
        };
    }
    apply_appearance(values.appearance, cx);
    cx.set_global(SettingsState {
        values: values.clone(),
        error,
    });
    cx.set_global(SettingsWrites {
        queue: Arc::new(Mutex::new(WriteQueue {
            committed: values,
            pending: None,
            running: false,
            error: None,
        })),
        save: Arc::new(move |values| values.save(&destination)),
        worker: None,
        notification: None,
    });
    cx.on_app_quit(|cx| {
        let writes = cx.global_mut::<SettingsWrites>();
        // Shutdown holds the App borrow. Cancel UI notifications before waiting
        // for the independent disk worker; it must never re-enter GPUI here.
        writes.notification.take();
        let worker = writes.worker.take();
        async move {
            if let Some(worker) = worker {
                worker.await;
            }
        }
    })
    .detach();
}

pub(super) struct WriteQueue {
    pub(super) committed: Preferences,
    pub(super) pending: Option<Preferences>,
    pub(super) running: bool,
    pub(super) error: Option<String>,
}

type SavePreferences = dyn Fn(&Preferences) -> voxelite_app::editor::EditorResult<()> + Send + Sync;

pub(super) struct SettingsWrites {
    pub(super) queue: Arc<Mutex<WriteQueue>>,
    pub(super) save: Arc<SavePreferences>,
    pub(super) worker: Option<Task<()>>,
    pub(super) notification: Option<Task<()>>,
}
impl Global for SettingsWrites {}

// Only disk work and the bounded queue live on this worker. Taking the next
// snapshot and transitioning to idle share a lock, so no edit can be stranded.
pub(super) fn drain_writes(queue: &Mutex<WriteQueue>, save: &SavePreferences) {
    loop {
        let pending = {
            let mut queue = queue.lock().unwrap_or_else(|e| e.into_inner());
            match queue.pending.take() {
                Some(pending) => pending,
                None => {
                    queue.running = false;
                    return;
                }
            }
        };
        let result = save(&pending);
        let mut queue = queue.lock().unwrap_or_else(|e| e.into_inner());
        match result {
            Ok(()) => {
                queue.committed = pending;
                queue.error = None;
            }
            Err(error) => {
                queue.error = Some(format!("Settings were not saved: {error}"));
                eprintln!("Settings were not saved: {error}");
            }
        }
    }
}

fn apply_appearance(appearance: Appearance, cx: &App) {
    cx.set_window_appearance(match appearance {
        Appearance::System => None,
        Appearance::Light => Some(WindowAppearance::Light),
        Appearance::Dark => Some(WindowAppearance::Dark),
    });
}

pub(in crate::ui) fn change(cx: &mut App, edit: impl FnOnce(&mut Preferences)) {
    let mut state = cx.global::<SettingsState>().clone();
    edit(&mut state.values);
    apply_appearance(state.values.appearance, cx);
    let next = state.values.clone();
    cx.set_global(state);
    let writes = cx.global_mut::<SettingsWrites>();
    {
        let mut queue = writes.queue.lock().unwrap_or_else(|e| e.into_inner());
        queue.pending = Some(next);
        if queue.running {
            return;
        }
        queue.running = true;
    }
    // Any prior notification describes an older batch. Read the latest durable
    // snapshot after this worker finishes, including edits coalesced during I/O.
    writes.notification.take();
    let queue = writes.queue.clone();
    let save = writes.save.clone();
    let (done, receive) = futures_channel::oneshot::channel();
    let worker = cx.background_executor().spawn(async move {
        drain_writes(&queue, &*save);
        let _ = done.send(());
    });
    let notification = cx.spawn(async move |cx| {
        if receive.await.is_err() {
            return;
        }
        cx.update(|cx| {
            let (values, error) = {
                let writes = cx.global::<SettingsWrites>();
                let queue = writes.queue.lock().unwrap_or_else(|e| e.into_inner());
                (queue.committed.clone(), queue.error.clone())
            };
            apply_appearance(values.appearance, cx);
            cx.update_global::<SettingsState, _>(|state, _| {
                state.values = values;
                state.error = error;
            });
        });
    });
    let writes = cx.global_mut::<SettingsWrites>();
    writes.worker = Some(worker);
    writes.notification = Some(notification);
}
