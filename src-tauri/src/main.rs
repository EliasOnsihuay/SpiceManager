fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            spicemanager::commands::detect_state,
            spicemanager::commands::install_all,
            spicemanager::commands::update_all,
            spicemanager::commands::repair_all,
            spicemanager::commands::validate_all,
            spicemanager::commands::recover_compatibility,
            spicemanager::commands::doctor_report,
            spicemanager::commands::export_diagnostics,
            spicemanager::commands::app_update_check,
            spicemanager::commands::app_update_download,
            spicemanager::commands::app_update_status,
            spicemanager::commands::current_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running SpiceManager");
}
