#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod reader;
use reader::{Payload, Reader};
use std::{
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
};
use tauri_plugin_dialog::DialogExt;

struct State {
    reader: Mutex<Reader>,
    pending: Mutex<Option<PathBuf>>,
    started: Instant,
    ready: AtomicBool,
    clipboard: Mutex<Option<arboard::Clipboard>>,
}

#[tauri::command]
fn copy_text(state: tauri::State<'_, State>, text: String) -> Result<(), String> {
    let mut clipboard = state
        .clipboard
        .lock()
        .map_err(|_| "clipboard is unavailable")?;
    if clipboard.is_none() {
        *clipboard = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
    }
    clipboard
        .as_mut()
        .unwrap()
        .set_text(text)
        .map_err(|e| e.to_string())
}

// Opt-in local measurement only; the path comes from the launching process,
// never from document content or JavaScript. Normal launches write no metrics.
#[derive(serde::Deserialize, serde::Serialize)]
struct FrontendTimings {
    ipc_ms: f64,
    template_ms: f64,
    decoration_ms: f64,
    chunks_ms: f64,
    attach_ms: f64,
    heading_index_ms: f64,
    display_ms: f64,
    focus_ms: f64,
    outline_index_ms: f64,
    outline_paint_ms: f64,
    outline_ms: f64,
    recents_ms: f64,
    scroll_ms: f64,
    progress_ms: f64,
    finish_ms: f64,
    total_ms: f64,
}

#[tauri::command]
fn reader_ready(
    state: tauri::State<'_, State>,
    timings: Option<FrontendTimings>,
) -> Result<(), String> {
    if let Some(path) = std::env::var_os("MARKLIGHT_BENCH_OUTPUT")
        && !state.ready.swap(true, Ordering::Relaxed)
    {
        let report = serde_json::json!({"ready_ms":state.started.elapsed().as_secs_f64()*1000.0,"pid":std::process::id(),"frontend":timings});
        std::fs::write(
            path,
            serde_json::to_vec(&report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn open_document(app: tauri::AppHandle, path: String, dark: bool) -> Result<Payload, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<State>();
        let mut reader = state
            .reader
            .lock()
            .map_err(|_| "reader state is unavailable")?;
        let events = app.clone();
        reader
            .open(std::path::Path::new(&path), dark, move || {
                let _ = events.emit("document-changed", ());
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn reload_document(app: tauri::AppHandle, dark: bool) -> Result<Payload, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<State>()
            .reader
            .lock()
            .map_err(|_| "reader state is unavailable")?
            .reload(dark)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn choose_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Markdown", &["md", "markdown"])
            .blocking_pick_file()
            .map(|file| {
                file.into_path()
                    .map(|p| p.to_string_lossy().into_owned())
                    .map_err(|e| e.to_string())
            })
            .transpose()
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_config(state: tauri::State<'_, State>) -> Result<marklight_core::Config, String> {
    Ok(state
        .reader
        .lock()
        .map_err(|_| "reader state is unavailable")?
        .config
        .clone())
}

#[tauri::command]
fn save_preferences(
    state: tauri::State<'_, State>,
    theme: marklight_core::Theme,
    font_size: u8,
    toc: bool,
    zen_mode: bool,
) -> Result<(), String> {
    let mut reader = state
        .reader
        .lock()
        .map_err(|_| "reader state is unavailable")?;
    let mut next = reader.config.clone();
    next.theme = theme;
    next.font_size = font_size;
    next.toc = toc;
    next.zen_mode = zen_mode;
    reader.save_config(next).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_recent(state: tauri::State<'_, State>) -> Result<(), String> {
    let mut reader = state
        .reader
        .lock()
        .map_err(|_| "reader state is unavailable")?;
    let mut next = reader.config.clone();
    next.recent.clear();
    reader.save_config(next).map_err(|e| e.to_string())
}

#[tauri::command]
fn take_pending(state: tauri::State<'_, State>) -> Result<Option<String>, String> {
    Ok(state
        .pending
        .lock()
        .map_err(|_| "pending state is unavailable")?
        .take()
        .map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
async fn follow_link(app: tauri::AppHandle, href: String) -> Result<reader::Navigation, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let navigation = app
            .state::<State>()
            .reader
            .lock()
            .map_err(|_| "reader state is unavailable")?
            .navigate(&href)
            .map_err(|e| e.to_string())?;
        if let reader::Navigation::External { url } = &navigation {
            tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())?;
        }
        Ok(navigation)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn queue_open(app: &tauri::AppHandle, path: PathBuf) {
    if let Ok(mut pending) = app.state::<State>().pending.lock() {
        *pending = Some(path);
    }
    let _ = app.emit("open-request", ());
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn main() {
    let started = Instant::now();
    let store = match marklight_core::ConfigStore::platform() {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Marklight: {e}");
            return;
        }
    };
    let initial = std::env::args_os()
        .nth(1)
        .filter(|p| !p.to_string_lossy().starts_with('-'))
        .map(PathBuf::from);
    let app = tauri::Builder::default()
        .manage(State {
            reader: Mutex::new(Reader::new(store)),
            pending: Mutex::new(initial),
            started,
            ready: AtomicBool::new(false),
            clipboard: Mutex::new(None),
        })
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if let Some(path) = args.get(1).filter(|p| !p.starts_with('-')) {
                queue_open(app, PathBuf::from(cwd).join(path));
            } else if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_document,
            reload_document,
            choose_file,
            get_config,
            save_preferences,
            clear_recent,
            take_pending,
            follow_link,
            reader_ready,
            copy_text
        ])
        .register_uri_scheme_protocol("marklight-image", |context, request| {
            let token = request.uri().path().trim_start_matches('/');
            let image = context
                .app_handle()
                .state::<State>()
                .reader
                .lock()
                .ok()
                .and_then(|reader| reader.image(token).ok());
            match image {
                Some((bytes, mime)) => tauri::http::Response::builder()
                    .header("Content-Type", mime)
                    .header("X-Content-Type-Options", "nosniff")
                    .header("Access-Control-Allow-Origin", "*")
                    .body(bytes)
                    .unwrap(),
                None => tauri::http::Response::builder()
                    .status(404)
                    .body(Vec::new())
                    .unwrap(),
            }
        })
        .setup(|app| {
            let open = MenuItem::with_id(app, "open", "Open…", true, Some("CmdOrCtrl+O"))?;
            let find =
                MenuItem::with_id(app, "find", "Find in Document…", true, Some("CmdOrCtrl+F"))?;
            let outline = MenuItem::with_id(
                app,
                "outline",
                "Toggle Outline",
                true,
                Some("CmdOrCtrl+Shift+T"),
            )?;
            let zen = MenuItem::with_id(
                app,
                "zen",
                "Toggle Zen Mode",
                true,
                Some("CmdOrCtrl+Shift+Z"),
            )?;
            let larger = MenuItem::with_id(
                app,
                "larger",
                "Increase Text Size",
                true,
                Some("CmdOrCtrl+="),
            )?;
            let smaller = MenuItem::with_id(
                app,
                "smaller",
                "Decrease Text Size",
                true,
                Some("CmdOrCtrl+-"),
            )?;
            let reset =
                MenuItem::with_id(app, "reset", "Reset Text Size", true, Some("CmdOrCtrl+0"))?;
            let menu = Menu::with_items(
                app,
                &[
                    &Submenu::with_items(
                        app,
                        "Marklight",
                        true,
                        &[
                            &PredefinedMenuItem::about(app, Some("About Marklight"), None)?,
                            &PredefinedMenuItem::quit(app, None)?,
                        ],
                    )?,
                    &Submenu::with_items(
                        app,
                        "File",
                        true,
                        &[&open, &PredefinedMenuItem::close_window(app, None)?],
                    )?,
                    &Submenu::with_items(
                        app,
                        "Edit",
                        true,
                        &[
                            &PredefinedMenuItem::copy(app, None)?,
                            &PredefinedMenuItem::paste(app, None)?,
                            &PredefinedMenuItem::select_all(app, None)?,
                        ],
                    )?,
                    &Submenu::with_items(
                        app,
                        "View",
                        true,
                        &[
                            &find,
                            &outline,
                            &zen,
                            &PredefinedMenuItem::separator(app)?,
                            &larger,
                            &smaller,
                            &reset,
                        ],
                    )?,
                ],
            )?;
            app.set_menu(menu)?;
            app.on_menu_event(|app, event| {
                if event.id().as_ref() == "open" {
                    let _ = app.emit("menu-open", ());
                } else if ["find", "outline", "zen", "larger", "smaller", "reset"]
                    .contains(&event.id().as_ref())
                {
                    let _ = app.emit("reader-action", event.id().as_ref());
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = event
                && let Some(path) = urls.into_iter().find_map(|url| url.to_file_path().ok())
            {
                queue_open(app, path);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        }),
        Err(e) => eprintln!("Marklight: unable to start desktop reader: {e}"),
    }
}
