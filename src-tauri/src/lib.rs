mod category_node;
mod preset;
mod product;

use category_node::CategoryNode;
use platform_dirs::AppDirs;
use preset::Preset;
use product::Product;
use rusqlite::{Connection, OpenFlags};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
use tauri::{Manager, State};

struct AppState {
    db: Option<Connection>,
    categories: CategoryNode,
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
            get_vendors
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

            app.manage(Mutex::new(AppState {
                db: conn,
                categories,
            }));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
