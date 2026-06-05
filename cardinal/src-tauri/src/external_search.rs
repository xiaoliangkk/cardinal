use crate::window_controls::activate_window;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Url};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSearchPayload {
    pub query: String,
}

static PENDING_EXTERNAL_SEARCH: Lazy<Mutex<Option<ExternalSearchPayload>>> =
    Lazy::new(|| Mutex::new(None));

#[tauri::command]
pub fn take_pending_external_search() -> Option<ExternalSearchPayload> {
    PENDING_EXTERNAL_SEARCH.lock().take()
}

#[tauri::command]
pub fn clear_pending_external_search() {
    PENDING_EXTERNAL_SEARCH.lock().take();
}

pub fn handle_opened_urls(app_handle: &AppHandle, urls: Vec<Url>) {
    for url in urls {
        let Some(payload) = parse_external_search_url(&url) else {
            continue;
        };
        dispatch_external_search(app_handle, payload);
    }
}

fn parse_external_search_url(url: &Url) -> Option<ExternalSearchPayload> {
    if url.scheme() != "cardinal" || url.host_str() != Some("search") {
        return None;
    }

    let query = url
        .query_pairs()
        .find(|(key, _)| key == "query" || key == "q")
        .map(|(_, value)| value.into_owned())?;
    let query = query.trim().to_string();
    if query.is_empty() {
        return None;
    }

    Some(ExternalSearchPayload { query })
}

fn dispatch_external_search(app_handle: &AppHandle, payload: ExternalSearchPayload) {
    *PENDING_EXTERNAL_SEARCH.lock() = Some(payload.clone());

    let Some(window) = app_handle.get_webview_window("main") else {
        warn!("External search requested before main window is available");
        return;
    };

    activate_window(&window);
    if let Err(err) = window.emit("external_search", payload) {
        warn!(?err, "Failed to emit external search event");
    } else {
        info!("External search event emitted");
    }
}
