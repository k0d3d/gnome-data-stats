# Gnome Data Stats

A modern, lightweight network data usage monitor for GNOME and other Linux desktops. Built with **Tauri**, **React**, and **Rust**.

![Gnome Data Stats](https://raw.githubusercontent.com/tauri-apps/tauri/dev/.github/app-screenshot.png) *(Placeholder for your screenshot)*

## ✨ Features

- **Real-time Monitoring:** Live upload and download speeds in the UI and System Tray.
- **Session Tracking:** Tracks how much data you've transferred since opening the app.
- **Granular History:** Automatically logs usage to a local SQLite database with **Hourly, Daily, and Monthly** breakdowns.
- **Robust Persistence:** Statistics are recorded by the backend even when the UI is closed.
- **Interface Filtering:** View usage history filtered by individual network interfaces.
- **Data Plan Heuristics:** Set a monthly limit and track your daily progress against it.
- **GNOME Aesthetic:** Designed with an Adwaita-inspired (GTK-4) look and feel.
- **System Tray Integration:** Quick-glance speeds in your top bar.
- **Low Footprint:** High performance and low memory usage thanks to Rust.

## 🚀 Getting Started

### Prerequisites (Linux)
You need the following system dependencies to build the app:
```bash
sudo apt update && sudo apt install -y \
  libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev build-essential curl wget file libssl-dev pkg-config
```

### Development
1. Clone the repository.
2. Install dependencies: `pnpm install`
3. Start the dev server: `pnpm tauri dev`

### Build Production
To create a `.deb` or `AppImage` with the correct versioning:
```bash
pnpm build:release
```
The installers will be generated in `src-tauri/target/release/bundle/`.

## 📦 Versioning
This project uses `package.json` as the source of truth for versioning.
To synchronize versions across the project, use:
```bash
pnpm sync-version
```

## 🛠 Tech Stack
- **Frontend:** React 19, Vite, TypeScript, Vanilla CSS (Adwaita variables).
- **Backend:** Rust, Tauri v2.
- **Database:** SQLite (via `tauri-plugin-sql`).
- **System API:** `/proc/net/dev` for high-performance network polling.

## 📄 License
MIT © [Your Name/Organization]
