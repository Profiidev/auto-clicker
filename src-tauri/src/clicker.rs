use std::{
  sync::mpsc::{self, Sender},
  thread::{self, JoinHandle},
  time::Duration,
};

use device_query::{DeviceQuery, DeviceState, Keycode};
use mouse_rs::Mouse;
use tauri::{AppHandle, Emitter};

pub struct Clicker {
  thread: Option<JoinHandle<()>>,
  sender: Sender<Message>,
}

enum Message {
  Exit,
  NewCPS(f32),
  RecordKey
}

impl Clicker {
  pub fn new(app: AppHandle) -> Self {
    let (sender, receiver) = mpsc::channel();

    let thread = thread::spawn(move || {
      let mut clicking;
      let mouse: Mouse = Mouse::new();
      let keyboard = DeviceState::new();
      let mut cps = 10.0;
      let mut current_loop = 0;
      let mut record_key = false;
      let mut last_keys = Vec::new();
      let mut current_code = Vec::new();

      loop {
        let message = receiver.recv_timeout(Duration::from_millis(1));

        if let Ok(message) = message {
          match message {
            Message::Exit => break,
            Message::NewCPS(new_cps) => cps = new_cps,
            Message::RecordKey => record_key = true,
          };
        }

        let keys = keyboard.get_keys();
        clicking = current_code.iter().filter(|&k| keys.contains(k)).collect::<Vec<&Keycode>>().len() == current_code.len() && !current_code.is_empty();

        if record_key {
          if !last_keys.clone().into_iter().filter(|k| !keys.contains(k)).collect::<Vec<Keycode>>().is_empty() {
            current_code = last_keys;
            last_keys = Vec::new();
            record_key = false;
            app.emit("recorded", current_code.iter().map(|&k| k.to_string()).collect::<Vec<String>>()).unwrap();
          } else {
            last_keys = keys;
          }
        }

        if clicking {
          current_loop += 1;
          if current_loop as f32 >= 1000.0 / cps {
            current_loop = 0;
            mouse.click(&mouse_rs::types::keys::Keys::LEFT).unwrap();
          }
        }
      }
    });

    Self {
      thread: Some(thread),
      sender,
    }
  }

  pub fn end(&self) {
    self.sender.send(Message::Exit).unwrap();
  }

  pub fn set_cps(&self, cps: f32) {
    self.sender.send(Message::NewCPS(cps)).unwrap();
  }

  pub fn record_key(&self) {
    self.sender.send(Message::RecordKey).unwrap();
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
