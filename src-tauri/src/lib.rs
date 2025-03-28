mod category_node;
mod preset;
mod product;

use category_node::CategoryNode;
use platform_dirs::AppDirs;
use preset::Preset;
use product::Product;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use rusqlite::{Connection, OpenFlags};
use std::{collections::HashMap, fs::File, path::PathBuf, sync::Mutex};
use tauri::{
    Manager, State,
    async_runtime::{Sender, channel, spawn_blocking},
};

struct AppState {
    db: Option<Connection>,
    categories: CategoryNode,
    preview_sender: Sender<PathBuf>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn get_categories(state: State<'_, Mutex<AppState>>) -> CategoryNode {
    state.lock().unwrap().categories.clone()
}

#[tauri::command]
fn get_presets(
    state: State<'_, Mutex<AppState>>,
    vendors: Vec<String>,
    products: Vec<usize>,
    offset: usize,
    limit: usize,
) -> Vec<Preset> {
    let mut cmd: String = "\
SELECT k_sound_info.id, k_sound_info.name, k_sound_info.vendor, \
       k_sound_info.comment, k_content_path.alias \
FROM k_sound_info \
INNER JOIN k_content_path ON k_sound_info.content_path_id = k_content_path.id"
        .into();
    let mut where_clauses: Vec<String> = vec![];

    if !vendors.is_empty() {
        where_clauses.push(format!(
            "k_sound_info.vendor IN ({})",
            vendors
                .into_iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if !products.is_empty() {
        where_clauses.push(format!(
            "k_content_path.id IN ({})",
            products
                .into_iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if !where_clauses.is_empty() {
        cmd.push_str(&format!(" WHERE {}", where_clauses.join(" AND ")));
    }

    cmd.push_str(&format!(
        " ORDER BY name ASC LIMIT {} OFFSET {}",
        limit, offset
    ));

    let state = state.lock().unwrap();
    let db = state.db.as_ref().unwrap();
    let mut stmt = db.prepare(&cmd).unwrap();

    let presets = stmt
        .query_map([], |row| {
            Ok(Preset {
                id: row.get::<usize, usize>(0).unwrap(),
                name: row.get::<usize, String>(1).unwrap_or("".into()),
                vendor: row.get::<usize, String>(2).unwrap_or("".into()),
                comment: row.get::<usize, String>(3).unwrap_or("".into()),
                product: row.get::<usize, String>(4).unwrap_or("".into()),
            })
        })
        .unwrap()
        .filter_map(|p| p.ok())
        .collect::<Vec<_>>();

    presets
}

#[tauri::command]
fn get_products(state: State<'_, Mutex<AppState>>, vendors: Vec<String>) -> Vec<Product> {
    let state = state.lock().unwrap();
    let db = state.db.as_ref().unwrap();
    let mut map: HashMap<usize, String> = HashMap::new();
    let mut stmt = db
        .prepare(
            "\
SELECT id, alias FROM k_content_path \
WHERE alias != '' AND content_type = 2
",
        )
        .unwrap();

    let mut rows = stmt.query([]).unwrap();

    while let Some(row) = rows.next().unwrap() {
        map.insert(
            row.get::<usize, usize>(0).unwrap(),
            row.get::<usize, String>(1).unwrap(),
        );
    }

    drop(rows);

    let mut cmd: String = "\
SELECT DISTINCT content_path_id, vendor FROM k_sound_info \
WHERE vendor != ''"
        .into();

    if !vendors.is_empty() {
        cmd.push_str(&format!(
            " AND vendor IN ({})",
            vendors
                .into_iter()
                .map(|v| format!("'{}'", v))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    stmt = db.prepare(&cmd).unwrap();

    let mut rows = stmt.query([]).unwrap();
    let mut p: HashMap<usize, Product> = HashMap::new();

    while let Some(row) = rows.next().unwrap() {
        let id = row.get::<usize, usize>(0).unwrap();

        if !map.contains_key(&id) || p.contains_key(&id) {
            continue;
        }

        p.insert(
            id,
            Product {
                id,
                name: map.get(&id).unwrap().clone(),
                vendor: row.get::<usize, String>(1).unwrap(),
            },
        );
    }

    let mut p: Vec<Product> = p.into_values().collect::<Vec<_>>();

    p.sort();

    p
}

#[tauri::command]
fn get_vendors(state: State<'_, Mutex<AppState>>) -> Vec<String> {
    let state = state.lock().unwrap();
    let db = state.db.as_ref().unwrap();
    let mut stmt = db
        .prepare("SELECT DISTINCT vendor FROM k_sound_info")
        .unwrap();

    stmt.query_map([], |row| row.get::<usize, String>(0))
        .unwrap()
        .filter_map(|v| v.ok())
        .collect::<Vec<_>>()
}

#[tauri::command]
fn play_preset(state: State<'_, Mutex<AppState>>, preset: usize) {
    let state = state.lock().unwrap();
    let db = state.db.as_ref().unwrap();
    let mut stmt = db
        .prepare(&format!(
            "SELECT file_name, content_path_id FROM k_sound_info WHERE id = {}",
            preset
        ))
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    let row = rows.next().unwrap().unwrap();

    let patch_path = PathBuf::from(&row.get::<usize, String>(0).unwrap());

    let preview_path: Option<PathBuf> = {
        let p = patch_path.parent().unwrap().join(".previews").join(format!(
            "{}.ogg",
            patch_path.file_name().unwrap().to_str().unwrap()
        ));
        if p.exists() {
            Some(p)
        } else {
            // we need to search for the previews library by name instruments
            let json_path: PathBuf = if cfg!(target_os = "macos") {
                PathBuf::from(
                    "/Users/Shared/Native Instruments/installed_products/Native Browser Preview Library.json",
                )
            } else {
                PathBuf::from(
                    "C:/Users/Public/Documents/Native Instruments/installed_products/Native Browser Preview Library.json",
                )
            };

            if json_path.exists() {
                let content_path_id = row.get::<usize, usize>(1).unwrap();

                let file = File::open(json_path).unwrap();

                let json: serde_json::Value = serde_json::from_reader(&file).unwrap();

                let preview_content_dir = PathBuf::from(json["ContentDir"].as_str().unwrap());

                let mut stmt = db
                    .prepare(&format!(
                        "SELECT path, upid FROM k_content_path WHERE id = {}",
                        &content_path_id
                    ))
                    .unwrap();
                let mut rows = stmt.query([]).unwrap();
                let row = rows.next().unwrap().unwrap();

                let content_dir = row.get::<usize, String>(0).unwrap();
                if let Ok(upid) = row.get::<usize, String>(1) {
                    Some(
                        preview_content_dir
                            .join("Samples")
                            .join(&upid)
                            .join(patch_path.strip_prefix(content_dir).unwrap())
                            .parent()
                            .unwrap()
                            .join(".previews")
                            .join(format!(
                                "{}.ogg",
                                patch_path.file_name().unwrap().to_str().unwrap()
                            )),
                    )
                } else {
                    None
                }
            } else {
                None
            }
        }
    };

    if let Some(preview_path) = preview_path {
        if preview_path.exists() {
            state.preview_sender.blocking_send(preview_path).unwrap();
        }
    }
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
        .invoke_handler(tauri::generate_handler![
            db_found,
            get_categories,
            get_presets,
            get_products,
            get_vendors,
            play_preset,
        ])
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

            let mut categories = CategoryNode::new(None, "");

            if let Some(ref conn) = conn {
                let mut stmt = conn
                    .prepare("SELECT id, category, subcategory, subsubcategory FROM k_category")
                    .unwrap();

                let mut rows = stmt.query([]).unwrap();

                while let Some(row) = rows.next().unwrap() {
                    categories.append(
                        Some(row.get::<usize, usize>(0).unwrap()),
                        vec![
                            row.get::<usize, String>(1).unwrap(),
                            row.get::<usize, String>(2).unwrap_or("".into()),
                            row.get::<usize, String>(3).unwrap_or("".into()),
                        ],
                    );
                }
            }

            let (sender, mut receiver) = channel::<PathBuf>(10);

            app.manage(Mutex::new(AppState {
                db: conn,
                categories,
                preview_sender: sender,
            }));

            spawn_blocking(move || {
                let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
                let mixer = stream_handle.mixer();
                let sink = Sink::connect_new(mixer);

                while let Some(path) = receiver.blocking_recv() {
                    let file = File::open(path).unwrap();

                    if !sink.empty() {
                        sink.clear();
                    }
                    sink.append(Decoder::try_from(file).unwrap());
                    sink.play();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
