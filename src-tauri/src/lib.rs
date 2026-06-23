mod config;
mod project;
mod outline;
mod template;
mod style;
mod style_guide;
mod ai;
mod pdf;
mod error_log;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Config commands
            config::load_config,
            config::save_config,
            config::test_llm_connection,
            config::test_img_connection,
            // Project commands
            project::create_project,
            project::list_projects,
            project::open_project,
            project::delete_project,
            project::update_project_status,
            project::update_project_size_index,
            project::update_project_settings,
            project::split_pages,
            project::load_outline,
            project::list_pages,
            project::read_page,
            project::save_page,
            project::open_folder,
            project::open_project_folder,
            // Outline commands
            outline::generate_outline,
            outline::parse_outline,
            outline::save_outline,
            outline::load_prompt,
            outline::save_prompt,
            outline::regenerate_page,
            // Template commands
            template::list_templates,
            template::get_template_info,
            // Style commands
            style::list_styles,
            style::get_style_content,
            style::save_style,
            style::delete_style,
            // Style guide commands
            style_guide::get_style_guide,
            style_guide::update_style_adherence,
            style_guide::get_layout_assignments,
            style_guide::extract_style_guide_from_images,
            // AI commands
            ai::imggen::generate_all_images,
            ai::imggen::generate_image,
            ai::imggen::regenerate_image,
            ai::imggen::refine_image,
            // PDF commands
            pdf::export_pdf,
            // Log commands
            error_log::load_error_log,
            error_log::clear_error_log,
        ])
        .setup(|app| {
            // Get the working directory
            let cwd = std::env::current_dir().expect("Failed to get current directory");
            app.manage(std::sync::Arc::new(cwd));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
