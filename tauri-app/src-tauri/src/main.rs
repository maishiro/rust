// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, Window};


#[derive(serde::Serialize)]
struct CustomResponse {
  message: String,
  other_val: usize,
}

// the payload type must implement `Serialize` and `Clone`.
#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn do_process(window: Window) {
  let num = 321;
  window.emit( "event_b2f", num ).unwrap();
}


fn main() {
  tauri::Builder::default()
    .setup(|app| {
      #[cfg(debug_assertions)] // only include this code on debug builds
      {
        let window = app.get_window("main").unwrap();
        window.open_devtools();
        window.close_devtools();
      }

      // `main` here is the window label; it is defined on the window creation or under `tauri.conf.json`
      // the default value is `main`. note that it must be unique
      // let main_window = app.get_window("main").unwrap();

      // // listen to the `event-name` (emitted on the `main` window)
      // let id = main_window.listen("event-name", |event| {
      // println!("got window event-name with payload {:?}", event.payload());
      // });
      // // unlisten to the event using the `id` returned on the `listen` function
      // // an `once` API is also exposed on the `Window` struct
      // main_window.unlisten(id);

      // // emit the `event-name` event to the `main` window
      // main_window.emit("event-name", Payload { message: "Tauri is awesome!".into() }).unwrap();

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![greet,do_process])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
