use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};
use serde::Serialize;
use tauri::{Emitter, Manager, Runtime, AppHandle};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder};
use tauri_plugin_sql::{Migration, MigrationKind, DbInstances, DbPool};
use chrono::{Local, Datelike, Timelike};
use sqlx::sqlite::SqlitePool;
use std::sync::Mutex;

mod app_stats;
use app_stats::{AppUsage, get_process_map};

#[derive(Serialize, Clone, Debug)]
struct NetworkInterface {
    name: String,
    is_up: bool,
}

#[derive(Serialize, Clone, Debug)]
struct SpeedStats {
    interface: String,
    download_speed: f64,
    upload_speed: f64,
    session_download: u64,
    session_upload: u64,
}

#[derive(Serialize, Clone, Debug, sqlx::FromRow)]
struct HistoryEntry {
    period: String,
    interface: String,
    download: u64,
    upload: u64,
}

#[derive(Default)]
struct NetState {
    prev_bytes: HashMap<String, (u64, u64)>,
    last_update: Option<Instant>,
    accumulated: HashMap<String, (u64, u64)>,
    session_totals: HashMap<String, (u64, u64)>,
}

struct AppState {
    tracking_enabled: Mutex<bool>,
    app_usage: Mutex<HashMap<String, AppUsage>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            tracking_enabled: Mutex::new(false),
            app_usage: Mutex::new(HashMap::new()),
        }
    }
}

fn format_tray_speed(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1024.0 {
        format!("{:.0}B/s", bytes_per_sec)
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{:.1}K/s", bytes_per_sec / 1024.0)
    } else {
        format!("{:.1}M/s", bytes_per_sec / 1024.0 / 1024.0)
    }
}

fn get_raw_stats() -> HashMap<String, (u64, u64)> {
    let mut stats = HashMap::new();
    if let Ok(content) = fs::read_to_string("/proc/net/dev") {
        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 9 {
                let name = parts[0].trim_end_matches(':').to_string();
                let rx = parts[1].parse::<u64>().unwrap_or(0);
                let tx = parts[9].parse::<u64>().unwrap_or(0);
                stats.insert(name, (rx, tx));
            }
        }
    }
    stats
}

#[tauri::command]
fn get_network_interfaces() -> Vec<NetworkInterface> {
    let stats = get_raw_stats();
    stats.keys()
        .filter(|&name| name != "lo")
        .map(|name| NetworkInterface {
            name: name.clone(),
            is_up: true,
        })
        .collect()
}

async fn get_pool<R: Runtime>(handle: &AppHandle<R>) -> Result<SqlitePool, String> {
    let instances = handle.state::<DbInstances>();
    let instances_lock = instances.0.read().await;
    let db_pool = instances_lock.get("sqlite:stats.db").ok_or("Database not loaded")?;
    match db_pool {
        DbPool::Sqlite(pool) => Ok(pool.clone()),
        _ => Err("Expected SQLite database".to_string()),
    }
}

#[tauri::command]
async fn get_history<R: Runtime>(
    handle: AppHandle<R>,
    period_type: String, // "hourly", "daily", "monthly"
    interface: Option<String>,
    page: u32,
    per_page: u32,
) -> Result<Vec<HistoryEntry>, String> {
    let pool = get_pool(&handle).await?;
    let offset = page * per_page;

    let (sql, filter) = match period_type.as_str() {
        "hourly" => {
            if let Some(ref iface) = interface {
                ("SELECT time_period as period, interface, download, upload FROM hourly_stats WHERE interface = ? ORDER BY time_period DESC LIMIT ? OFFSET ?", Some(iface))
            } else {
                ("SELECT time_period as period, interface, download, upload FROM hourly_stats ORDER BY time_period DESC LIMIT ? OFFSET ?", None)
            }
        }
        "monthly" => {
            if let Some(ref iface) = interface {
                ("SELECT strftime('%Y-%m', day) as period, interface, SUM(download) as download, SUM(upload) as upload FROM daily_stats WHERE interface = ? GROUP BY period, interface ORDER BY period DESC LIMIT ? OFFSET ?", Some(iface))
            } else {
                ("SELECT strftime('%Y-%m', day) as period, interface, SUM(download) as download, SUM(upload) as upload FROM daily_stats GROUP BY period, interface ORDER BY period DESC LIMIT ? OFFSET ?", None)
            }
        }
        "daily" | _ => {
            if let Some(ref iface) = interface {
                ("SELECT day as period, interface, download, upload FROM daily_stats WHERE interface = ? ORDER BY day DESC LIMIT ? OFFSET ?", Some(iface))
            } else {
                ("SELECT day as period, interface, download, upload FROM daily_stats ORDER BY day DESC LIMIT ? OFFSET ?", None)
            }
        }
    };

    let mut query = sqlx::query_as::<_, HistoryEntry>(sql);
    if let Some(f) = filter {
        query = query.bind(f);
    }
    query = query.bind(per_page as i64).bind(offset as i64);

    let result = query
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
async fn toggle_app_tracking(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let mut enabled = state.tracking_enabled.lock().unwrap();
    *enabled = !*enabled;
    Ok(*enabled)
}

#[tauri::command]
async fn get_app_usage(state: tauri::State<'_, AppState>) -> Result<Vec<AppUsage>, String> {
    let usage = state.app_usage.lock().unwrap();
    Ok(usage.values().cloned().collect())
}

fn start_monitoring<R: Runtime>(app: &tauri::App<R>) {
    let handle = app.handle().clone();
    let app_state = app.state::<AppState>();
    let app_state_inner = app_state.inner().clone();

    // Main network monitoring thread
    std::thread::spawn(move || {
        let mut state = NetState::default();
        let mut save_counter = 0;

        loop {
            let current_stats = get_raw_stats();
            let now = Instant::now();

            if let Some(last_time) = state.last_update {
                let elapsed = now.duration_since(last_time).as_secs_f64();
                if elapsed > 0.0 {
                    let mut total_dl = 0.0;
                    let mut total_ul = 0.0;

                    for (name, (curr_rx, curr_tx)) in &current_stats {
                        if let Some(&(prev_rx, prev_tx)) = state.prev_bytes.get(name) {
                            let dl_delta = curr_rx.saturating_sub(prev_rx);
                            let ul_delta = curr_tx.saturating_sub(prev_tx);

                            let dl_speed = dl_delta as f64 / elapsed;
                            let ul_speed = ul_delta as f64 / elapsed;

                            total_dl += dl_speed;
                            total_ul += ul_speed;

                            let acc = state.accumulated.entry(name.clone()).or_insert((0, 0));
                            acc.0 += dl_delta;
                            acc.1 += ul_delta;

                            let sess = state.session_totals.entry(name.clone()).or_insert((0, 0));
                            sess.0 += dl_delta;
                            sess.1 += ul_delta;

                            let _ = handle.emit("network-speed", SpeedStats {
                                interface: name.clone(),
                                download_speed: dl_speed,
                                upload_speed: ul_speed,
                                session_download: sess.0,
                                session_upload: sess.1,
                            });
                        }
                    }

                    if let Some(tray) = handle.tray_by_id("main-tray") {
                        let tray_title = format!("↓{} ↑{}", format_tray_speed(total_dl), format_tray_speed(total_ul));
                        let _ = tray.set_title(Some(tray_title));
                    }
                }
            }

            state.prev_bytes = current_stats;
            state.last_update = Some(now);
            save_counter += 1;

            if save_counter >= 30 {
                let handle_clone = handle.clone();
                let accumulated = state.accumulated.clone();
                state.accumulated.clear();
                save_counter = 0;

                tauri::async_runtime::spawn(async move {
                    if let Ok(pool) = get_pool(&handle_clone).await {
                        let now = Local::now();
                        let day = now.format("%Y-%m-%d").to_string();
                        let hour = format!("{}-{:02}-{:02} {:02}:00", now.year(), now.month(), now.day(), now.hour());

                        for (iface, (rx, tx)) in accumulated {
                            if rx == 0 && tx == 0 { continue; }

                            let _ = sqlx::query(
                                "INSERT INTO daily_stats (day, interface, download, upload) 
                                 VALUES (?, ?, ?, ?) 
                                 ON CONFLICT(day, interface) DO UPDATE SET 
                                 download = download + excluded.download, 
                                 upload = upload + excluded.upload"
                            )
                            .bind(&day)
                            .bind(&iface)
                            .bind(rx as i64)
                            .bind(tx as i64)
                            .execute(&pool)
                            .await;

                            let _ = sqlx::query(
                                "INSERT INTO hourly_stats (time_period, interface, download, upload) 
                                 VALUES (?, ?, ?, ?) 
                                 ON CONFLICT(time_period, interface) DO UPDATE SET 
                                 download = download + excluded.download, 
                                 upload = upload + excluded.upload"
                            )
                            .bind(&hour)
                            .bind(&iface)
                            .bind(rx as i64)
                            .bind(tx as i64)
                            .execute(&pool)
                            .await;
                        }
                        let _ = handle_clone.emit("stats-saved", ());
                    }
                });
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    });

    // App monitoring thread
    let app_handle_for_stats = app.handle().clone();
    std::thread::spawn(move || {
        let mut last_io = HashMap::new();
        
        loop {
            let enabled = *app_state_inner.tracking_enabled.lock().unwrap();
            if enabled {
                if let Ok(all_procs) = procfs::process::all_processes() {
                    let mut current_usage = app_state_inner.app_usage.lock().unwrap();
                    
                    for p in all_procs {
                        if let Ok(proc) = p {
                            if let Ok(io) = proc.io() {
                                let pid = proc.pid();
                                let name = proc.stat().map(|s| s.comm).unwrap_or_else(|_| "unknown".to_string());
                                
                                let (prev_read, prev_write) = last_io.get(&pid).cloned().unwrap_or((io.read_bytes, io.write_bytes));
                                
                                let read_delta = io.read_bytes.saturating_sub(prev_read);
                                let write_delta = io.write_bytes.saturating_sub(prev_write);
                                
                                if read_delta > 0 || write_delta > 0 {
                                    let entry = current_usage.entry(name.clone()).or_insert(AppUsage {
                                        name: name.clone(),
                                        download: 0,
                                        upload: 0,
                                    });
                                    entry.download += read_delta;
                                    entry.upload += write_delta;
                                }
                                
                                last_io.insert(pid, (io.read_bytes, io.write_bytes));
                            }
                        }
                    }
                }
            } else {
                last_io.clear();
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create daily_stats table",
            sql: "CREATE TABLE IF NOT EXISTS daily_stats (
                day TEXT,
                interface TEXT,
                download INTEGER,
                upload INTEGER,
                PRIMARY KEY (day, interface)
            );",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create hourly_stats table",
            sql: "CREATE TABLE IF NOT EXISTS hourly_stats (
                time_period TEXT,
                interface TEXT,
                download INTEGER,
                upload INTEGER,
                PRIMARY KEY (time_period, interface)
            );",
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:stats.db", migrations)
                .build(),
        )
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            start_monitoring(app);
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_network_interfaces, 
            get_history, 
            toggle_app_tracking, 
            get_app_usage
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
