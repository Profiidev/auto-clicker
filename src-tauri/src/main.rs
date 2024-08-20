// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs, sync::{Arc, Mutex}};

use clicker::{Clicker, Config};
use tauri::{Manager, State};

mod clicker;

#[tauri::command]
fn cps(state: State<'_, AppState>, cps: f32) {
    let clicker = state.clicker.lock().unwrap();
    clicker.as_ref().unwrap().set_cps(cps);
}

#[tauri::command]
fn chance(state: State<'_, AppState>, chance: i32) {
    let clicker = state.clicker.lock().unwrap();
    clicker.as_ref().unwrap().set_random(chance);
}

#[tauri::command]
fn record(state: State<'_, AppState>) {
    let clicker = state.clicker.lock().unwrap();
    clicker.as_ref().unwrap().record_key();
}

#[tauri::command]
fn bind(state: State<'_, AppState>) -> Vec<String> {
    let bind = state.bind.lock().unwrap();
    if let Some(bind) = bind.as_ref() {
        bind.clone()
    } else {
        Vec::new()
    }
}

struct AppState {
    clicker: Arc<Mutex<Option<Clicker>>>,
    bind: Arc<Mutex<Option<Vec<String>>>>
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            clicker: Arc::new(Mutex::new(None)),
            bind: Arc::new(Mutex::new(None)),
        })
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle();
            let state = handle.state::<AppState>();

            let mut dir = app.path().app_config_dir().unwrap_or_default().into_os_string();
            dir.push("/config.json");
            let config = if let Ok(data) = fs::read(dir) {
                Some(serde_json::from_slice::<Config>(&data).unwrap())
            } else {
                None
            };

            if let Some(config) = &config {
                state.bind.lock().unwrap().replace(config.bind.clone());
            }
            state.clicker.lock().unwrap().replace(Clicker::new(app.handle().to_owned(), config, state.bind.clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![cps, record, chance, bind])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| if let tauri::RunEvent::ExitRequested { .. } = event {
                let state = app_handle.state::<AppState>();
                let clicker = state.clicker.lock().unwrap();
                clicker.as_ref().unwrap().end();
            });
}
