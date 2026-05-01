mod commands;
mod dto;
mod file_reader;
mod protocol;
mod serial_reader;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::list_serial_ports,
            commands::connect_serial,
            commands::start_demo,
            commands::disconnect,
            commands::send_reset,
            commands::current_source,
            commands::write_text_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running TRV Timer");
}
