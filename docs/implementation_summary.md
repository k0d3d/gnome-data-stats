# Implementation Summary: Gnome Data Stats

## 🛠 Progress Log

### Phase 1-4: Foundation & UI (Completed)
- Initialized Tauri v2 project with React 19 + Adwaita styling.
- Implemented real-time polling and System Tray integration.
- Established SQLite persistence for daily totals.

### Phase 5: Cumulative Monitoring & Data Plans (Completed)
- **Session Tracking:** Added Rust-side accumulation to track bytes transferred since app launch.
- **Daily Persistence:** Integrated frontend logic to fetch and display today's totals from SQLite.
- **Data Plan Heuristics:** 
  - Added a "Plan" tab to set monthly data limits (stored in `localStorage`).
  - Implemented a progress bar in the Live view to track daily usage against the monthly cap.
- **Interface Persistence:** The app now remembers the last selected network interface across restarts.
- **Improved UI:** Added a "Usage Grid" for quick glances at Session vs. Daily totals.

### Phase 6: Robust Persistence & Granular History (v0.2.0 - Completed)
- **Backend-side Persistence:** Moved the data recording logic from React to the Rust backend. Statistics are now saved to the database every 30 seconds, even when the UI is closed.
- **Granular History:** 
  - **Hourly Tracking:** Added a new `hourly_stats` table to track usage per hour.
  - **Monthly Aggregation:** Implemented backend logic to aggregate daily data into monthly summaries.
- **Advanced History Filtering:**
  - Added a period selector (Hourly, Daily, Monthly) to the History tab.
  - Implemented interface-specific filtering to view history for a single interface at a time.
  - Integrated an interface selector directly into the History view.
- **Database Migrations:** Implemented a versioned migration system to seamlessly upgrade existing databases (adding `hourly_stats` without data loss).

---

## 🔧 Technical Details: Traffic Differentiation
- **Heuristic Approach:** The app monitors all traffic on the selected interface.
- **Classification Note:** To maintain security and avoid requiring `root` privileges, the app currently tracks total interface throughput. An information panel was added to the "Plan" tab to explain how this relates to ISP data caps.
- **Direct SQL Access:** The app uses `tauri-plugin-sql` and `sqlx` in the Rust backend to manage data efficiently.

---

## 🔧 Critical Fixes & System Setup
- **Dependency Resolutions:** Fixed `tauri-plugin-opener` naming and resolved Tauri v2 `tray-icon` feature conflicts.
- **Backend Libraries:** Added `sqlx`, `chrono`, and `serde` to the Rust side to support advanced data handling.
- **Tauri Config:** Corrected the placement of the `plugins` section in `tauri.conf.json` for Tauri v2 compatibility.

---

## 📦 Versioning & Distribution
The project uses `package.json` as the source of truth for versioning.

### Synchronizing Versions
To keep `package.json`, `tauri.conf.json`, and `Cargo.toml` in sync, run:
```bash
pnpm sync-version
```

### Building the Release
To sync versions and build the production-ready installers in one command:
```bash
pnpm build:release
```

### Output Files
When building version `X.Y.Z`, Tauri automatically generates the files with the correct version in the name:
- **Debian (.deb):** `src-tauri/target/release/bundle/deb/gnome-data-stats_X.Y.Z_amd64.deb`
- **AppImage:** `src-tauri/target/release/bundle/appimage/gnome-data-stats_X.Y.Z_amd64.AppImage`
- **Binary:** `src-tauri/target/release/gnome-data-stats`
