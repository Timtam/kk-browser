mod category;
mod paginated_result;
mod preset;
mod product;

use category::{Category, Mode};
use multi_key_map::MultiKeyMap;
use ordered_hash_map::OrderedHashMap;
use paginated_result::PaginatedResult;
use platform_dirs::AppDirs;
use preset::Preset;
use product::{Product, ProductKey};
use rodio::{Decoder, OutputStreamBuilder, Sink};
use rusqlite::{Connection, OpenFlags};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    path::PathBuf,
    sync::Mutex,
};
use tauri::{
    Manager, State,
    async_runtime::{Sender, channel, spawn_blocking},
};

struct AppState {
    categories: HashMap<usize, Category>,
    db_found: bool,
    loading: bool,
    modes: HashMap<usize, Mode>,
    products: MultiKeyMap<ProductKey, Product>,
    presets: OrderedHashMap<usize, Preset>,
    preview_sender: Sender<PathBuf>,
    vendors: Vec<String>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn get_categories(
    state: State<'_, Mutex<AppState>>,
    vendors: Vec<String>,
    products: Vec<usize>,
) -> Vec<Category> {
    let products = products.into_iter().map(ProductKey::Id).collect::<Vec<_>>();
    let state = state.lock().unwrap();

    state
        .categories
        .values()
        .filter(|c| {
            (vendors.is_empty()
                || c.presets
                    .iter()
                    .any(|p| vendors.contains(&state.presets.get(p).unwrap().vendor)))
                && (products.is_empty()
                    || c.presets
                        .iter()
                        .any(|p| products.contains(&state.presets.get(p).unwrap().product_id)))
        })
        .cloned()
        .collect::<Vec<_>>()
}

#[tauri::command]
async fn get_presets(
    state: State<'_, Mutex<AppState>>,
    vendors: Vec<String>,
    products: Vec<usize>,
    categories: Vec<usize>,
    offset: usize,
    limit: usize,
) -> Result<PaginatedResult<Preset>, ()> {
    let products: Vec<ProductKey> = products.into_iter().map(ProductKey::Id).collect::<Vec<_>>();
    let state = state.lock().unwrap();

    let presets: Vec<Preset> = state
        .presets
        .values()
        .filter(|p| {
            (vendors.is_empty() || vendors.contains(&p.vendor))
                && (products.is_empty() || products.contains(&p.product_id))
                && (categories.is_empty() || categories.iter().any(|c| p.categories.contains(c)))
        })
        .cloned()
        .collect::<Vec<_>>();

    let results = presets
        .iter()
        .skip(offset)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();

    let start = offset + 1;
    let end = offset + results.len();

    Ok(PaginatedResult {
        results,
        start,
        end,
        total: presets.len(),
    })
}

#[tauri::command]
async fn get_products(
    state: State<'_, Mutex<AppState>>,
    vendors: Vec<String>,
    categories: Vec<usize>,
) -> Result<Vec<Product>, ()> {
    let state = state.lock().unwrap();

    let mut p: Vec<Product> = state
        .products
        .values()
        .filter(|p| {
            (vendors.is_empty() || vendors.contains(&p.vendor))
                && (categories.is_empty()
                    || state
                        .presets
                        .values()
                        .any(|p| p.categories.iter().any(|c| categories.contains(c))))
        })
        .cloned()
        .collect::<Vec<_>>();

    p.sort();

    Ok(p)
}

#[tauri::command]
fn get_vendors(state: State<'_, Mutex<AppState>>) -> Vec<String> {
    state.lock().unwrap().vendors.clone()
}

#[tauri::command]
fn play_preset(state: State<'_, Mutex<AppState>>, preset: usize) {
    let state = state.lock().unwrap();
    let preset = state.presets.get(&preset).unwrap();

    let preview_path: Option<PathBuf> = {
        let p = preset
            .file_name
            .parent()
            .unwrap()
            .join(".previews")
            .join(format!(
                "{}.ogg",
                preset.file_name.file_name().unwrap().to_str().unwrap()
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
                let file = File::open(json_path).unwrap();

                let json: serde_json::Value = serde_json::from_reader(&file).unwrap();

                let preview_content_dir = PathBuf::from(json["ContentDir"].as_str().unwrap());

                let product = state.products.get(&preset.product_id).unwrap();

                if !product.upid.is_empty() {
                    Some(
                        preview_content_dir
                            .join("Samples")
                            .join(&product.upid)
                            .join(preset.file_name.strip_prefix(&product.content_dir).unwrap())
                            .parent()
                            .unwrap()
                            .join(".previews")
                            .join(format!(
                                "{}.ogg",
                                preset.file_name.file_name().unwrap().to_str().unwrap()
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
    state.lock().unwrap().db_found
}

#[tauri::command]
fn is_loading(state: State<'_, Mutex<AppState>>) -> bool {
    state.lock().unwrap().loading
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
            is_loading,
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

            let (sender, mut receiver) = channel::<PathBuf>(10);

            app.manage(Mutex::new(AppState {
                db_found: conn.is_some(),
                loading: true,
                categories: HashMap::new(),
                modes: HashMap::new(),
                products: MultiKeyMap::new(),
                presets: OrderedHashMap::new(),
                preview_sender: sender,
                vendors: vec![],
            }));

            if let Some(conn) = conn {
                let handle = app.app_handle().clone();

                spawn_blocking(move || {
                    let state = handle.state::<Mutex<AppState>>();
                    let mut stmt = conn
                        .prepare("SELECT DISTINCT vendor FROM k_sound_info")
                        .unwrap();

                    let vendors: Vec<String> = stmt
                        .query_map([], |row| row.get::<usize, String>(0))
                        .unwrap()
                        .filter_map(|v| v.ok())
                        .collect::<Vec<_>>();

                    let mut products: MultiKeyMap<ProductKey, Product> = MultiKeyMap::new();

                    let mut map: HashMap<usize, (String, String, String)> = HashMap::new();
                    let mut stmt = conn
                        .prepare(
                            "\
SELECT id, path, alias, upid FROM k_content_path",
                        )
                        .unwrap();

                    let mut rows = stmt.query([]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        map.insert(
                            row.get::<usize, usize>(0).unwrap(),
                            (
                                row.get::<usize, String>(1).unwrap(),
                                row.get::<usize, String>(2).unwrap_or("".into()),
                                row.get::<usize, String>(3).unwrap_or("".into()),
                            ),
                        );
                    }

                    drop(rows);

                    let cmd: String = "\
SELECT DISTINCT content_path_id, vendor FROM k_sound_info"
                        .into();

                    stmt = conn.prepare(&cmd).unwrap();

                    let mut rows = stmt.query([]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        let id = row.get::<usize, usize>(0).unwrap();

                        if !map.contains_key(&id) || products.contains_key(&ProductKey::Id(id)) {
                            continue;
                        }

                        let keys: Vec<ProductKey> = match map.get(&id).unwrap().2.as_str() {
                            "" => vec![ProductKey::Id(id)],
                            other => vec![ProductKey::Id(id), ProductKey::Upid(other.to_string())],
                        };

                        products.insert_many(
                            keys,
                            Product {
                                id,
                                name: map.get(&id).unwrap().1.clone(),
                                vendor: row.get::<usize, String>(1).unwrap_or("".into()),
                                content_dir: map.get(&id).unwrap().0.clone(),
                                upid: map.get(&id).unwrap().2.clone(),
                            },
                        );
                    }

                    let mut presets: OrderedHashMap<usize, Preset> = OrderedHashMap::new();

                    let cmd: String = "\
SELECT \
    id, name, vendor, comment, content_path_id, file_name \
FROM k_sound_info"
                        .into();

                    let mut stmt = conn.prepare(&cmd).unwrap();

                    let mut p: Vec<Preset> = stmt
                        .query_map([], |row| {
                            Ok(Preset {
                                id: row.get::<usize, usize>(0).unwrap(),
                                name: row.get::<usize, String>(1).unwrap_or("".into()),
                                vendor: row.get::<usize, String>(2).unwrap_or("".into()),
                                comment: row.get::<usize, String>(3).unwrap_or("".into()),
                                product_id: ProductKey::Id(row.get::<usize, usize>(4).unwrap()),
                                product_name: products
                                    .get(&ProductKey::Id(row.get::<usize, usize>(4).unwrap()))
                                    .unwrap()
                                    .name
                                    .clone(),
                                file_name: PathBuf::from(&row.get::<usize, String>(5).unwrap()),
                                categories: HashSet::new(),
                                modes: HashSet::new(),
                            })
                        })
                        .unwrap()
                        .filter_map(|p| p.ok())
                        .collect::<Vec<_>>();

                    p.sort();

                    p.into_iter().for_each(|p| {
                        presets.insert(p.id, p);
                    });

                    let mut categories: HashMap<usize, Category> = HashMap::new();

                    let mut stmt = conn
                        .prepare("SELECT id, category, subcategory, subsubcategory FROM k_category")
                        .unwrap();

                    let mut rows = stmt.query([]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        categories.insert(
                            row.get::<usize, usize>(0).unwrap(),
                            Category {
                                id: row.get::<usize, usize>(0).unwrap(),
                                name: row.get::<usize, String>(1).unwrap(),
                                subcategory: row.get::<usize, String>(2).unwrap_or("".into()),
                                subsubcategory: row.get::<usize, String>(3).unwrap_or("".into()),
                                presets: HashSet::new(),
                            },
                        );
                    }

                    let mut stmt = conn
                        .prepare("SELECT sound_info_id, category_id FROM k_sound_info_category")
                        .unwrap();

                    let mut rows = stmt.query([]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        categories
                            .entry(row.get::<usize, usize>(1).unwrap())
                            .and_modify(|c| {
                                c.presets.insert(row.get::<usize, usize>(0).unwrap());
                            });
                        presets
                            .get_mut(&row.get::<usize, usize>(0).unwrap())
                            .unwrap()
                            .categories
                            .insert(row.get::<usize, usize>(1).unwrap());
                    }

                    let mut modes: HashMap<usize, Mode> = HashMap::new();

                    let mut stmt = conn.prepare("SELECT id, name FROM k_mode").unwrap();

                    let mut rows = stmt.query([]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        modes.insert(
                            row.get::<usize, usize>(0).unwrap(),
                            Mode {
                                id: row.get::<usize, usize>(0).unwrap(),
                                name: row.get::<usize, String>(1).unwrap(),
                                presets: HashSet::new(),
                            },
                        );
                    }

                    let mut stmt = conn
                        .prepare("SELECT sound_info_id, mode_id FROM k_sound_info_mode")
                        .unwrap();

                    let mut rows = stmt.query([]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        modes
                            .entry(row.get::<usize, usize>(1).unwrap())
                            .and_modify(|m| {
                                m.presets.insert(row.get::<usize, usize>(0).unwrap());
                            });
                        presets
                            .get_mut(&row.get::<usize, usize>(0).unwrap())
                            .unwrap()
                            .modes
                            .insert(row.get::<usize, usize>(1).unwrap());
                    }

                    let mut locked_state = state.lock().unwrap();

                    locked_state.vendors = vendors;
                    locked_state.categories = categories;
                    locked_state.modes = modes;
                    locked_state.products = products;
                    locked_state.presets = presets;
                    locked_state.loading = false;
                });
            }

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
