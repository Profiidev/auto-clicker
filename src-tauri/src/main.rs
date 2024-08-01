// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{sync::{mpsc::{self, Sender}, Arc, Mutex}, thread::{self, sleep, JoinHandle}, time::Duration};

use mouse_rs::Mouse;
use tauri::{Manager, State};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn start(state: State<'_, AppState>) {
    let clicker = state.clicker.lock().unwrap();
    clicker.start();
}

#[tauri::command]
fn stop(state: State<'_, AppState>) {
    let clicker = state.clicker.lock().unwrap();
    clicker.stop();
}

struct AppState {
    clicker: Arc<Mutex<Clicker>>
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            clicker: Arc::new(Mutex::new(Clicker::new()))
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![start, stop])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| if let tauri::RunEvent::ExitRequested { .. } = event {
                app_handle.state::<AppState>().clicker.lock().unwrap().end();
            });
}

struct Clicker {
    thread: Option<JoinHandle<()>>,
    sender: Sender<Message>
}

enum Message {
    Exit,
    Start,
    Stop
}

impl Clicker {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let thread = thread::spawn(move || {
            let mut clicking = false;
            let mouse = Mouse::new();
            loop {
                let message = receiver.recv_timeout(Duration::from_millis(1));
                if let Ok(message) = message {
                    match message {
                        Message::Exit => break,
                        Message::Start => clicking = true,
                        Message::Stop => clicking = false,
                    };
                }

                sleep(Duration::from_millis(50));
                if clicking {
                    mouse.click(&mouse_rs::types::keys::Keys::LEFT).unwrap();
                }
            }
        });
        
        Self {
            thread: Some(thread),
            sender
        }
    }

    pub fn start(&self) {
        self.sender.send(Message::Start).unwrap();
    }

    pub fn stop(&self) {
        self.sender.send(Message::Stop).unwrap();
    }

    pub fn end(&self) {
        self.sender.send(Message::Exit).unwrap();
    }
}

impl Drop for Clicker {
    fn drop(&mut self) {
        self.sender.send(Message::Exit).unwrap();
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap()
        }
    }
}
