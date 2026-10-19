mod commands;
mod core;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(core::Service::new(app.path().app_data_dir()?.join("snapshots")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::accounts,
            commands::dota_running,
            commands::bundled_presets,
            commands::read_preset,
            commands::write_preset,
            commands::bindings,
            commands::export_current,
            commands::preview,
            commands::apply,
            commands::snapshots,
            commands::create_snapshot,
            commands::restore_snapshot,
            commands::delete_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
