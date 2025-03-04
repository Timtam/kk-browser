use platform_dirs::AppDirs;
use rusqlite::{Connection, OpenFlags};
use std::{path::PathBuf, sync::Mutex};
use tauri::{Manager, State};

struct AppState {
    db: Option<Connection>,
    vendors: Vec<String>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn get_vendors(state: State<'_, Mutex<AppState>>) -> Vec<String> {
    state.lock().unwrap().vendors.clone()
}

#[tauri::command]
fn db_found(state: State<'_, Mutex<AppState>>) -> bool {
    state.lock().unwrap().db.is_some()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![db_found, get_vendors])
        .setup(|app| {
            let db3_path: PathBuf = AppDirs::new(Some("Native Instruments"), true)
                .unwrap()
                .cache_dir
                .join(PathBuf::from("Komplete Kontrol/Browser Data/komplete.db3"));

            let conn = Connection::open_with_flags(
                db3_path.as_path(),
                OpenFlags::SQLITE_OPEN_READ_ONLY
                    | OpenFlags::SQLITE_OPEN_URI
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .ok();

            let vendors: Vec<String> = if let Some(ref conn) = conn {
                let mut stmt = conn
                    .prepare("SELECT DISTINCT vendor FROM k_sound_info")
                    .unwrap();

                stmt.query_map([], |row| row.get::<usize, String>(0))
                    .unwrap()
                    .filter_map(|v| v.ok())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            };

            app.manage(Mutex::new(AppState { db: conn, vendors }));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
