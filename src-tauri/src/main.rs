// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use clicker::Clicker;
use tauri::{Manager, State};

mod clicker;

#[tauri::command]
fn cps(state: State<'_, AppState>, cps: f32) {
    let clicker = state.clicker.lock().unwrap();
    clicker.as_ref().unwrap().set_cps(cps);
}

#[tauri::command]
fn record(state: State<'_, AppState>) {
    let clicker = state.clicker.lock().unwrap();
    clicker.as_ref().unwrap().record_key();
}

struct AppState {
    clicker: Arc<Mutex<Option<Clicker>>>
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            clicker: Arc::new(Mutex::new(None))
        })
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle();
            let state = handle.state::<AppState>();
            state.clicker.lock().unwrap().replace(Clicker::new(app.handle().to_owned()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![cps, record])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| if let tauri::RunEvent::ExitRequested { .. } = event {
                let state = app_handle.state::<AppState>();
                let clicker = state.clicker.lock().unwrap();
                clicker.as_ref().unwrap().end();
            });
}
