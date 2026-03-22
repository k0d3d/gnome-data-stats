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

### Phase 7: App Tracking & UX Refinements (v0.3.1 - Completed)
- **Single Instance Enforcement:** Added the `tauri-plugin-single-instance` to prevent multiple app instances from running simultaneously.
- **History Pagination:** Added pagination (20 items per page) to the history view to improve performance with large datasets.
- **Detailed App Tracking (Privileged):**
  - Added a toggle in the "Live" tab to enable per-application network monitoring.
  - Implemented the `toggle_app_tracking` command to handle Polkit/root authentication triggers.
  - Integrated `procfs` and `pnet` crates in the backend to map sockets to specific processes.
- **About Menu:** Added a dedicated "About" tab displaying the application version, description, and project links.
- **UI Polishing:** Added toggle switches and styled the per-app usage list with Adwaita-consistent visuals.

---

## 🔧 Technical Details: Traffic Differentiation
- **Heuristic Approach:** The app monitors all traffic on the selected interface.
- **Classification Note:** To maintain security and avoid requiring `root` privileges by default, the app tracks total interface throughput. Detailed app tracking is opt-in and requires root elevation via Polkit.

---

## 🔧 Critical Fixes & System Setup
- **Dependency Resolutions:** Fixed `tauri-plugin-opener` naming and resolved Tauri v2 `tray-icon` feature conflicts.
- **Backend Libraries:** Added `sqlx`, `chrono`, `pnet`, `procfs`, and `tauri-plugin-single-instance` to the Rust side.
- **Tauri Config:** Corrected the placement of the `plugins` section in `tauri.conf.json`.

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

### GitHub Actions Release
To automate a release on GitHub, update the version in `package.json` and run:
```bash
pnpm release
```
This will sync versions, tag the commit, and push to the `master` branch to trigger the build workflow.
