# GNOME Data Stats - Tech Specs & Development Plan

## Project Overview
**GNOME Data Stats** is a lightweight, modern data usage monitoring application built with Tauri and React. It aims to provide real-time network statistics and historical data consumption tracking specifically optimized for GNOME-based Linux distributions.

## Technical Specifications

### Frontend
- **Framework:** React 19 (TypeScript)
- **Build Tool:** Vite
- **Styling:** Vanilla CSS (Adwaita-inspired design)
- **Charts:** (Optional) Lightweight SVG-based charts or `recharts` for historical visualization.

### Backend (Rust/Tauri)
- **System Integration:**
  - **Primary Source:** NetworkManager D-Bus API via `zbus`. This is the most reliable way on GNOME to get interface statuses and usage.
  - **Secondary Source:** `/proc/net/dev` for raw byte counters if D-Bus is unavailable.
- **Persistence:** `tauri-plugin-sql` (SQLite) to store daily, weekly, and monthly data usage logs.
- **Background Task:** A Rust-based "emitter" that polls network stats every 1-2 seconds and sends events to the frontend.

### Key Features
1. **Real-time Monitoring:** Live upload/download speeds per active interface.
2. **Session Tracking:** Bytes transferred since the application started.
3. **Daily Usage:** Persistent tracking of total bytes used per day using SQLite.
4. **Data Plan Settings:** User-defined monthly limits with visual progress indicators.
5. **Interface Memory:** Automatically restores the last used interface on startup.
6. **System Tray:** GNOME-style status indicator showing live speeds in the top bar.

---

## Development Plan

### Phase 1: Foundation & Data Access
- [ ] Initialize `docs/` and setup basic folder structure.
- [ ] Add necessary Rust dependencies: `zbus`, `serde`, `tauri-plugin-sql`.
- [ ] Implement a basic Rust function to read `/proc/net/dev` or D-Bus stats.
- [ ] Create a Tauri command to fetch the list of available network interfaces.

### Phase 2: Real-time Engine
- [ ] Set up a background thread in `src-tauri/src/lib.rs` to poll stats.
- [ ] Implement Tauri events to "push" speed updates to the React frontend.
- [ ] Build the core React components: `SpeedGauge`, `InterfaceSelector`, and `LiveGraph`.

### Phase 3: Persistence & History
- [ ] Configure `tauri-plugin-sql`.
- [ ] Implement a schema for daily data totals.
- [ ] Create a "History" view in React to visualize usage over the last 30 days.

### Phase 4: UI Polish & GNOME Integration
- [ ] Refine CSS to match GNOME's Adwaita/GTK-4 aesthetic (dark/light mode support).
- [ ] Implement system tray support for real-time speed display.
- [ ] Add user settings for data caps and notification alerts.

### Phase 5: Finalization & Testing
- [ ] Verify performance impact (CPU/Memory usage should be minimal).
- [ ] Package the app for Linux (AppImage/Debian).
