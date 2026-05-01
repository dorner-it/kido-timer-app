use tauri::{AppHandle, State};

use crate::state::AppState;

#[tauri::command]
pub fn list_serial_ports() -> Vec<String> {
    crate::serial_reader::list_ports()
}

#[tauri::command]
pub fn connect_serial(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    port: String,
    baud: u32,
) -> Result<(), String> {
    state.connect_serial(app_handle, port, baud)
}

#[tauri::command]
pub fn start_demo(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    path: String,
    speed: f64,
) -> Result<(), String> {
    state.start_demo(app_handle, path, speed)
}

#[tauri::command]
pub fn disconnect(state: State<'_, AppState>) {
    state.disconnect();
}

#[tauri::command]
pub fn send_reset(state: State<'_, AppState>) -> Result<(), String> {
    state.send_reset()
}

#[tauri::command]
pub fn current_source(state: State<'_, AppState>) -> Option<String> {
    state.connection.lock().as_ref().map(|c| c.source().to_string())
}

#[tauri::command]
pub fn write_text_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents.as_bytes()).map_err(|e| format!("{}: {}", path, e))
}
