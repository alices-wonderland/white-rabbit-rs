#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

#[tauri::command]
async fn add(a: i32, b: i32) -> i32 {
  backend_core::add(a, b)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  let _ = env_logger::try_init();

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![add])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  Ok(())
}
