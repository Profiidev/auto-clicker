use std::{
  fs, sync::{mpsc::{self, Sender}, Arc, Mutex}, thread::{self, JoinHandle}, time::Duration
};

use device_query::{DeviceQuery, DeviceState};
use mouse_rs::Mouse;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

pub struct Clicker {
  thread: Option<JoinHandle<()>>,
  sender: Sender<Message>,
}

enum Message {
  Exit,
  NewCPS(f32),
  NewRandom(i32),
  RecordKey
}

#[derive(Serialize, Deserialize)]
pub struct Config {
  cps: f32,
  chance: i32,
  pub bind: Vec<String>
}

impl Clicker {
  pub fn new(app: AppHandle, config: Option<Config>, bind: Arc<Mutex<Option<Vec<String>>>>) -> Self {
    let (sender, receiver) = mpsc::channel();

    let thread = thread::spawn(move || {
      let mut clicking;
      let mouse: Mouse = Mouse::new();
      let device = DeviceState::new();
      let mut cps = 10.0;
      let mut chance = 100;
      let mut current_loop = 0;
      let mut record_key = false;
      let mut last_keys = Vec::new();
      let mut current_code = Vec::new();
      let mut rng = thread_rng();

      if let Some(config) = config {
        cps = config.cps;
        chance = config.chance;
        current_code = config.bind;
      }

      loop {
        let message = receiver.recv_timeout(Duration::from_millis(1));

        if let Ok(message) = message {
          match message {
            Message::Exit => {
              let config = Config {
                cps,
                chance,
                bind: current_code
              };
              let config = serde_json::to_string(&config).unwrap();
              let mut dir = app.path().app_config_dir().unwrap().into_os_string();
              fs::create_dir_all(&dir).unwrap();
              dir.push("/config.json");
              fs::write(dir, config).unwrap();
              break;
            },
            Message::NewCPS(new_cps) => cps = new_cps,
            Message::NewRandom(new_random) => chance = new_random,
            Message::RecordKey => record_key = true,
          };
        }

        let keys = get_keys(&device);
        clicking = current_code.iter().filter(|&k| keys.contains(k)).collect::<Vec<&String>>().len() == current_code.len() && !current_code.is_empty();

        if record_key {
          if !last_keys.clone().into_iter().filter(|k| !keys.contains(k)).collect::<Vec<String>>().is_empty() {
            current_code = last_keys;
            last_keys = Vec::new();
            record_key = false;
            bind.lock().unwrap().replace(current_code.clone());
            app.emit("recorded", &current_code).unwrap();
          } else {
            last_keys = keys;
          }
        }

        if clicking {
          current_loop += 1;
          if current_loop as f32 >= 1000.0 / cps {
            current_loop = 0;
            let rand = rng.gen_range(0..100);
            if rand < chance {
              mouse.click(&mouse_rs::types::keys::Keys::LEFT).unwrap();
            }
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

  pub fn set_random(&self, random: i32) {
    self.sender.send(Message::NewRandom(random)).unwrap();
  }

  pub fn record_key(&self) {
    self.sender.send(Message::RecordKey).unwrap();
  }
}

impl Drop for Clicker {
  fn drop(&mut self) {
    self.sender.send(Message::Exit).unwrap();
    if let Some(thread) = self.thread.take() {
      thread.join().unwrap();
    }
  }
}

fn get_keys(device: &DeviceState) -> Vec<String> {
  let mut keyboard = device.get_keys().iter().map(|&k| k.to_string()).collect::<Vec<String>>();
  let mut mouse = Vec::new();
  for (i, b) in device.get_mouse().button_pressed.into_iter().enumerate() {
    if b {
      mouse.push(format!("Mouse{}", i));
    }
  }

  keyboard.append(&mut mouse);
  keyboard
}