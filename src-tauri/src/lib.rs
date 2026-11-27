mod services;
mod projects;
mod system;
mod vhosts;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            services::get_service_status,
            services::start_all,
            services::stop_all,
            services::start_service,
            services::stop_service,
            services::reload_service,
            system::quick_action,
            system::open_config,
            vhosts::sync_vhosts,
            projects::list_projects,
            projects::create_project,
            projects::open_project,
            projects::open_project_folder,
            projects::open_project_terminal
        ])
        .run(tauri::generate_context!())
        .expect("error while running Archagon");
}
