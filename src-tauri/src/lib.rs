use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};
use serde::Serialize;
use tauri::{Emitter, Manager, Runtime, AppHandle};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIcon};
use tauri_plugin_sql::{Migration, MigrationKind};

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
}

#[derive(Serialize, Clone, Debug)]
struct HistoryEntry {
    day: String,
    interface: String,
    download: u64,
    upload: u64,
}

#[derive(Default)]
struct NetState {
    prev_bytes: HashMap<String, (u64, u64)>,
    last_update: Option<Instant>,
    accumulated: HashMap<String, (u64, u64)>,
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

#[tauri::command]
async fn get_history(_app: AppHandle) -> Result<Vec<HistoryEntry>, String> {
    Ok(vec![])
}

fn start_monitoring<R: Runtime>(app: &tauri::App<R>) {
    let handle = app.handle().clone();
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

                            let _ = handle.emit("network-speed", SpeedStats {
                                interface: name.clone(),
                                download_speed: dl_speed,
                                upload_speed: ul_speed,
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

            // Every 30 seconds, emit an event for the frontend to save the accumulated data to SQL
            // This is a reliable pattern in Tauri when using frontend-centric plugins
            if save_counter >= 30 {
                let _ = handle.emit("save-stats", state.accumulated.clone());
                state.accumulated.clear();
                save_counter = 0;
            }

            std::thread::sleep(Duration::from_secs(1));
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
        }
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
        .invoke_handler(tauri::generate_handler![get_network_interfaces, get_history])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
