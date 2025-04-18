mod category;
mod paginated_result;
mod preset;
mod product;

use category::{Bank, Category, Mode};
use directories::BaseDirs;
use multi_key_map::MultiKeyMap;
use ordered_hash_map::OrderedHashMap;
use paginated_result::PaginatedResult;
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
    banks: OrderedHashMap<usize, Bank>,
    categories: OrderedHashMap<usize, Category>,
    db_found: bool,
    loading: bool,
    modes: OrderedHashMap<usize, Mode>,
    products: MultiKeyMap<ProductKey, Product>,
    presets: OrderedHashMap<usize, Preset>,
    preview_sender: Sender<PathBuf>,
    vendors: Vec<String>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
async fn get_categories(
    state: State<'_, Mutex<AppState>>,
    vendors: Vec<String>,
    products: Vec<usize>,
    modes: Vec<usize>,
    banks: Vec<usize>,
) -> Result<Vec<Category>, ()> {
    let products = products.into_iter().map(ProductKey::Id).collect::<Vec<_>>();
    let state = state.lock().unwrap();

    Ok(state
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
                && (modes.is_empty()
                    || c.presets.iter().any(|p| {
                        modes
                            .iter()
                            .any(|m| state.presets.get(p).unwrap().modes.contains(m))
                    }))
                && (banks.is_empty()
                    || c.presets
                        .iter()
                        .any(|p| banks.contains(&state.presets.get(p).unwrap().bank)))
        })
        .cloned()
        .collect::<Vec<_>>())
}

#[tauri::command]
async fn get_banks(
    state: State<'_, Mutex<AppState>>,
    vendors: Vec<String>,
    products: Vec<usize>,
    modes: Vec<usize>,
    categories: Vec<usize>,
) -> Result<Vec<Bank>, ()> {
    let products = products.into_iter().map(ProductKey::Id).collect::<Vec<_>>();
    let state = state.lock().unwrap();

    Ok(state
        .banks
        .values()
        .filter(|b| {
            (vendors.is_empty()
                || b.presets
                    .iter()
                    .any(|p| vendors.contains(&state.presets.get(p).unwrap().vendor)))
                && (products.is_empty()
                    || b.presets
                        .iter()
                        .any(|p| products.contains(&state.presets.get(p).unwrap().product_id)))
                && (modes.is_empty()
                    || b.presets.iter().any(|p| {
                        modes
                            .iter()
                            .any(|m| state.presets.get(p).unwrap().modes.contains(m))
                    }))
                && (categories.is_empty()
                    || b.presets.iter().any(|p| {
                        categories
                            .iter()
                            .any(|c| state.presets.get(p).unwrap().categories.contains(c))
                    }))
        })
        .cloned()
        .collect::<Vec<_>>())
}

#[tauri::command]
async fn get_modes(
    state: State<'_, Mutex<AppState>>,
    vendors: Vec<String>,
    products: Vec<usize>,
    categories: Vec<usize>,
    banks: Vec<usize>,
) -> Result<Vec<Mode>, ()> {
    let products = products.into_iter().map(ProductKey::Id).collect::<Vec<_>>();
    let state = state.lock().unwrap();

    Ok(state
        .modes
        .values()
        .filter(|m| {
            (vendors.is_empty()
                || m.presets
                    .iter()
                    .any(|p| vendors.contains(&state.presets.get(p).unwrap().vendor)))
                && (products.is_empty()
                    || m.presets
                        .iter()
                        .any(|p| products.contains(&state.presets.get(p).unwrap().product_id)))
                && (categories.is_empty()
                    || m.presets.iter().any(|p| {
                        categories
                            .iter()
                            .any(|c| state.presets.get(p).unwrap().categories.contains(c))
                    }))
                && (banks.is_empty()
                    || m.presets
                        .iter()
                        .any(|p| banks.contains(&state.presets.get(p).unwrap().bank)))
        })
        .cloned()
        .collect::<Vec<_>>())
}

#[tauri::command]
async fn get_presets(
    state: State<'_, Mutex<AppState>>,
    vendors: Vec<String>,
    products: Vec<usize>,
    categories: Vec<usize>,
    modes: Vec<usize>,
    banks: Vec<usize>,
    mut query: String,
    offset: usize,
    limit: usize,
) -> Result<PaginatedResult<Preset>, ()> {
    query = query.to_lowercase();
    let products: Vec<ProductKey> = products.into_iter().map(ProductKey::Id).collect::<Vec<_>>();
    let state = state.lock().unwrap();

    let presets: Vec<Preset> = state
        .presets
        .values()
        .filter(|p| {
            (vendors.is_empty() || vendors.contains(&p.vendor))
                && (products.is_empty() || products.contains(&p.product_id))
                && (categories.is_empty() || categories.iter().any(|c| p.categories.contains(c)))
                && (modes.is_empty() || modes.iter().any(|m| p.modes.contains(m)))
                && (banks.is_empty() || banks.contains(&p.bank))
                && (query.is_empty()
                    || p.name.to_lowercase().contains(&query)
                    || p.comment.to_lowercase().contains(&query))
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
    modes: Vec<usize>,
    banks: Vec<usize>,
) -> Result<Vec<Product>, ()> {
    let state = state.lock().unwrap();

    let mut p: Vec<Product> = state
        .products
        .values()
        .filter(|p| {
            (vendors.is_empty() || vendors.contains(&p.vendor))
                && (categories.is_empty()
                    || categories.iter().any(|c| {
                        p.presets
                            .iter()
                            .any(|pr| state.presets.get(pr).unwrap().categories.contains(c))
                    }))
                && (modes.is_empty()
                    || modes.iter().any(|m| {
                        p.presets
                            .iter()
                            .any(|pr| state.presets.get(pr).unwrap().modes.contains(m))
                    }))
                && (banks.is_empty()
                    || p.presets
                        .iter()
                        .any(|pr| banks.contains(&state.presets.get(pr).unwrap().bank)))
        })
        .cloned()
        .collect::<Vec<_>>();

    p.sort();

    Ok(p)
}

#[tauri::command]
async fn get_vendors(
    state: State<'_, Mutex<AppState>>,
    products: Vec<usize>,
    categories: Vec<usize>,
    modes: Vec<usize>,
    banks: Vec<usize>,
) -> Result<Vec<String>, ()> {
    let state = state.lock().unwrap();
    Ok(state
        .vendors
        .iter()
        .filter(|v| {
            (products.is_empty()
                || products
                    .iter()
                    .any(|p| &state.products.get(&ProductKey::Id(*p)).unwrap().vendor == *v))
                && (categories.is_empty()
                    || categories.iter().any(|c| {
                        state
                            .categories
                            .get(c)
                            .unwrap()
                            .presets
                            .iter()
                            .any(|p| &state.presets.get(p).unwrap().vendor == *v)
                    }))
                && (modes.is_empty()
                    || modes.iter().any(|m| {
                        state
                            .modes
                            .get(m)
                            .unwrap()
                            .presets
                            .iter()
                            .any(|p| &state.presets.get(p).unwrap().vendor == *v)
                    }))
                && (banks.is_empty()
                    || banks.iter().any(|b| {
                        state
                            .banks
                            .get(b)
                            .unwrap()
                            .presets
                            .iter()
                            .any(|pr| &state.presets.get(pr).unwrap().vendor == *v)
                    }))
        })
        .cloned()
        .collect::<Vec<_>>())
}

#[tauri::command]
fn play_preset(state: State<'_, Mutex<AppState>>, preset: usize) {
    let state = state.lock().unwrap();
    let preset = state.presets.get(&preset).unwrap();

    let preview_path: Option<PathBuf> = if preset
        .file_name
        .extension()
        .unwrap()
        .eq_ignore_ascii_case("wav")
        && preset.file_name.exists()
    {
        Some(preset.file_name.clone())
    } else {
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

fn get_db3_path() -> PathBuf {
    BaseDirs::new()
        .unwrap()
        .data_local_dir()
        .join(PathBuf::from(
            "Native Instruments/Komplete Kontrol/Browser Data/komplete.db3",
        ))
}

#[tauri::command]
fn get_db_path() -> String {
    get_db3_path().into_os_string().into_string().unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            db_found,
            get_banks,
            get_categories,
            get_db_path,
            get_modes,
            get_presets,
            get_products,
            get_vendors,
            is_loading,
            play_preset,
        ])
        .setup(|app| {
            let db3_path: PathBuf = get_db3_path();
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
                banks: OrderedHashMap::new(),
                categories: OrderedHashMap::new(),
                modes: OrderedHashMap::new(),
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

                    let mut banks: OrderedHashMap<usize, Bank> = OrderedHashMap::new();

                    let mut stmt = conn
                        .prepare("SELECT id, entry1, entry2, entry3 FROM k_bank_chain")
                        .unwrap();

                    let mut b: Vec<Bank> = stmt
                        .query_map([], |row| {
                            Ok(Bank {
                                id: row.get::<usize, usize>(0).unwrap(),
                                entry1: row.get::<usize, String>(1).unwrap(),
                                entry2: row.get::<usize, String>(2).unwrap_or("".into()),
                                entry3: row.get::<usize, String>(3).unwrap_or("".into()),
                                presets: HashSet::new(),
                            })
                        })
                        .unwrap()
                        .filter_map(|b| b.ok())
                        .collect::<Vec<_>>();

                    b.sort();

                    b.into_iter().for_each(|b| {
                        banks.insert(b.id, b);
                    });

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
                                presets: HashSet::new(),
                            },
                        );
                    }

                    let mut presets: OrderedHashMap<usize, Preset> = OrderedHashMap::new();

                    let cmd: String = "\
SELECT \
    id, name, vendor, comment, content_path_id, file_name, bank_chain_id \
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
                                bank: row.get::<usize, usize>(6).unwrap_or(0),
                            })
                        })
                        .unwrap()
                        .filter_map(|p| p.ok())
                        .collect::<Vec<_>>();

                    p.sort();

                    p.into_iter().for_each(|p| {
                        products
                            .get_mut(&p.product_id)
                            .unwrap()
                            .presets
                            .insert(p.id);
                        if p.bank != 0 {
                            banks.get_mut(&p.bank).unwrap().presets.insert(p.id);
                        }
                        presets.insert(p.id, p);
                    });

                    let mut categories: OrderedHashMap<usize, Category> = OrderedHashMap::new();

                    let mut stmt = conn
                        .prepare("SELECT id, category, subcategory, subsubcategory FROM k_category")
                        .unwrap();

                    let mut c: Vec<Category> = stmt
                        .query_map([], |row| {
                            Ok(Category {
                                id: row.get::<usize, usize>(0).unwrap(),
                                name: row.get::<usize, String>(1).unwrap(),
                                subcategory: row.get::<usize, String>(2).unwrap_or("".into()),
                                subsubcategory: row.get::<usize, String>(3).unwrap_or("".into()),
                                presets: HashSet::new(),
                            })
                        })
                        .unwrap()
                        .filter_map(|c| c.ok())
                        .collect::<Vec<_>>();

                    c.sort();

                    c.into_iter().for_each(|c| {
                        categories.insert(c.id, c);
                    });

                    let mut stmt = conn
                        .prepare("SELECT sound_info_id, category_id FROM k_sound_info_category")
                        .unwrap();

                    let mut rows = stmt.query([]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        categories
                            .get_mut(&row.get::<usize, usize>(1).unwrap())
                            .unwrap()
                            .presets
                            .insert(row.get::<usize, usize>(0).unwrap());
                        presets
                            .get_mut(&row.get::<usize, usize>(0).unwrap())
                            .unwrap()
                            .categories
                            .insert(row.get::<usize, usize>(1).unwrap());
                    }

                    let mut modes: OrderedHashMap<usize, Mode> = OrderedHashMap::new();

                    let mut stmt = conn.prepare("SELECT id, name FROM k_mode").unwrap();

                    let mut m: Vec<Mode> = stmt
                        .query_map([], |row| {
                            Ok(Mode {
                                id: row.get::<usize, usize>(0).unwrap(),
                                name: row.get::<usize, String>(1).unwrap(),
                                presets: HashSet::new(),
                            })
                        })
                        .unwrap()
                        .filter_map(|m| m.ok())
                        .collect::<Vec<_>>();

                    m.sort();

                    m.into_iter().for_each(|m| {
                        modes.insert(m.id, m);
                    });

                    let mut stmt = conn
                        .prepare("SELECT sound_info_id, mode_id FROM k_sound_info_mode")
                        .unwrap();

                    let mut rows = stmt.query([]).unwrap();

                    while let Some(row) = rows.next().unwrap() {
                        modes
                            .get_mut(&row.get::<usize, usize>(1).unwrap())
                            .unwrap()
                            .presets
                            .insert(row.get::<usize, usize>(0).unwrap());
                        presets
                            .get_mut(&row.get::<usize, usize>(0).unwrap())
                            .unwrap()
                            .modes
                            .insert(row.get::<usize, usize>(1).unwrap());
                    }

                    let mut locked_state = state.lock().unwrap();

                    locked_state.vendors = vendors;
                    locked_state.banks = banks;
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
