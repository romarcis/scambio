#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use rqs_lib::channel::{ChannelAction, ChannelDirection, ChannelMessage};
use rqs_lib::{EndpointInfo, OutboundPayload, SendInfo, Visibility, RQS};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::{broadcast, mpsc};

struct AppState {
    rqs: Mutex<RQS>,
    sender_file: mpsc::Sender<SendInfo>,
    endpoints: broadcast::Sender<EndpointInfo>,
    download_dir: Mutex<PathBuf>,
    mica: Mutex<bool>,
}

#[derive(Serialize)]
struct Info {
    name: String,
    download_dir: String,
    mica: bool,
}

fn default_download_dir() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|d| d.download_dir().map(PathBuf::from))
        .unwrap_or_else(std::env::temp_dir)
}

#[tauri::command]
fn info(state: State<'_, AppState>) -> Info {
    Info {
        name: std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC".into()),
        download_dir: state.download_dir.lock().unwrap().display().to_string(),
        mica: *state.mica.lock().unwrap(),
    }
}

#[tauri::command]
fn start_discovery(state: State<'_, AppState>) -> Result<(), String> {
    let mut rqs = state.rqs.lock().unwrap();
    rqs.stop_discovery();
    rqs.discovery(state.endpoints.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_discovery(state: State<'_, AppState>) {
    state.rqs.lock().unwrap().stop_discovery();
}

#[tauri::command]
async fn send_files(
    id: String,
    name: String,
    addr: String,
    files: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .sender_file
        .send(SendInfo {
            id,
            name,
            addr,
            ob: OutboundPayload::Files(files),
        })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn respond(id: String, action: ChannelAction, state: State<'_, AppState>) -> Result<(), String> {
    state
        .rqs
        .lock()
        .unwrap()
        .message_sender
        .send(ChannelMessage {
            id,
            direction: ChannelDirection::FrontToLib,
            action: Some(action),
            ..Default::default()
        })
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_visible(visible: bool, state: State<'_, AppState>) {
    let v = if visible { Visibility::Visible } else { Visibility::Invisible };
    state.rqs.lock().unwrap().change_visibility(v);
}

#[tauri::command]
async fn pick_files(app: AppHandle) -> Vec<String> {
    app.dialog()
        .file()
        .set_title("Scegli i file da inviare")
        .blocking_pick_files()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|f| f.into_path().ok())
        .map(|p| p.display().to_string())
        .collect()
}

#[tauri::command]
async fn pick_download_dir(app: AppHandle, state: State<'_, AppState>) -> Result<Option<String>, String> {
    let Some(dir) = app
        .dialog()
        .file()
        .set_title("Dove salvare i file ricevuti")
        .blocking_pick_folder()
        .and_then(|f| f.into_path().ok())
    else {
        return Ok(None);
    };
    state.rqs.lock().unwrap().set_download_path(Some(dir.clone()));
    *state.download_dir.lock().unwrap() = dir.clone();
    Ok(Some(dir.display().to_string()))
}

#[tauri::command]
fn open_download_dir(state: State<'_, AppState>) {
    let dir = state.download_dir.lock().unwrap().clone();
    let _ = std::process::Command::new("explorer").arg(dir).spawn();
}

/// Keeps Mica and the page's colour scheme on the same Windows theme.
fn apply_theme(w: &tauri::WebviewWindow, theme: tauri::Theme) -> bool {
    let _ = w.set_theme(Some(theme));
    window_vibrancy::apply_mica(w, Some(theme == tauri::Theme::Dark)).is_ok()
}

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let download_dir = default_download_dir();
    let mut rqs = RQS::new(Visibility::Visible, None, Some(download_dir.clone()));
    let (sender_file, _ble) = rqs.run().await.expect("Quick Share service failed to start");
    let messages = rqs.message_sender.clone();
    let (endpoints, _) = broadcast::channel(32);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            rqs: Mutex::new(rqs),
            sender_file,
            endpoints: endpoints.clone(),
            download_dir: Mutex::new(download_dir),
            mica: Mutex::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            info,
            start_discovery,
            stop_discovery,
            send_files,
            respond,
            set_visible,
            pick_files,
            pick_download_dir,
            open_download_dir,
        ])
        .setup(move |app| {
            // Mica needs Windows 11; on Windows 10 the UI falls back to a solid background.
            if let Some(w) = app.get_webview_window("main") {
                let ok = apply_theme(&w, w.theme().unwrap_or(tauri::Theme::Light));
                *app.state::<AppState>().mica.lock().unwrap() = ok;
            }

            let handle = app.handle().clone();
            let mut rx = messages.subscribe();
            tauri::async_runtime::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(m) if m.direction == ChannelDirection::LibToFront => {
                            let _ = handle.emit("transfer", m);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        _ => {}
                    }
                }
            });

            let handle = app.handle().clone();
            let mut rx = endpoints.subscribe();
            tauri::async_runtime::spawn(async move {
                loop {
                    match rx.recv().await {
                        Ok(e) => {
                            let _ = handle.emit("device", e);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        _ => {}
                    }
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::ThemeChanged(theme) = event {
                if let Some(w) = window.app_handle().get_webview_window("main") {
                    apply_theme(&w, *theme);
                }
            }
            if let tauri::WindowEvent::Destroyed = event {
                let state = window.app_handle().state::<AppState>();
                state.rqs.lock().unwrap().stop_discovery();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Scambio");
}
