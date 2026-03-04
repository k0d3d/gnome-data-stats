import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Database from "@tauri-apps/plugin-sql";
import "./App.css";

interface NetworkInterface {
  name: string;
  is_up: boolean;
}

interface SpeedStats {
  interface: string;
  download_speed: number;
  upload_speed: number;
}

interface HistoryEntry {
  day: string;
  interface: string;
  download: number;
  upload: number;
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec === 0) return "0.0 B/s";
  const units = ["B/s", "KB/s", "MB/s", "GB/s"];
  let i = 0;
  let speed = bytesPerSec;
  while (speed >= 1024 && i < units.length - 1) {
    speed /= 1024;
    i++;
  }
  return `${speed.toFixed(1)} ${units[i]}`;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let val = bytes;
  while (val >= 1024 && i < units.length - 1) {
    val /= 1024;
    i++;
  }
  return `${val.toFixed(2)} ${units[i]}`;
}

function App() {
  const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
  const [selected, setSelected] = useState<string>(() => {
    return localStorage.getItem("last-interface") || "";
  });
  const [tab, setTab] = useState<"live" | "history">("live");
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [speeds, setSpeeds] = useState<{ download: number; upload: number }>({
    download: 0,
    upload: 0,
  });

  useEffect(() => {
    async function fetchInterfaces() {
      try {
        const result = await invoke<NetworkInterface[]>("get_network_interfaces");
        setInterfaces(result);
        
        // Restore saved interface if it exists, otherwise pick first
        const saved = localStorage.getItem("last-interface");
        if (saved && result.some(i => i.name === saved)) {
          setSelected(saved);
        } else if (result.length > 0) {
          const defaultIface = result[0].name;
          setSelected(defaultIface);
          localStorage.setItem("last-interface", defaultIface);
        }
      } catch (error) {
        console.error("Failed to fetch interfaces:", error);
      }
    }
    fetchInterfaces();
  }, []);

  const handleInterfaceChange = (name: string) => {
    setSelected(name);
    localStorage.setItem("last-interface", name);
    // Reset speeds when changing interface to avoid stale data
    setSpeeds({ download: 0, upload: 0 });
  };

  useEffect(() => {
    const unlisten = listen<SpeedStats>("network-speed", (event) => {
      if (event.payload.interface === selected) {
        setSpeeds({
          download: event.payload.download_speed,
          upload: event.payload.upload_speed,
        });
      }
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [selected]);

  useEffect(() => {
    const unlistenSave = listen<Record<string, [number, number]>>("save-stats", async (event) => {
      const db = await Database.load("sqlite:stats.db");
      const day = new Date().toISOString().split("T")[0];
      
      for (const [iface, [rx, tx]] of Object.entries(event.payload)) {
        if (rx === 0 && tx === 0) continue;
        
        await db.execute(
          `INSERT INTO daily_stats (day, interface, download, upload) 
           VALUES (?, ?, ?, ?) 
           ON CONFLICT(day, interface) DO UPDATE SET 
           download = download + excluded.download, 
           upload = upload + excluded.upload`,
          [day, iface, rx, tx]
        );
      }
    });

    return () => {
      unlistenSave.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (tab === "history") {
      async function loadHistory() {
        try {
          const db = await Database.load("sqlite:stats.db");
          const result = await db.select<HistoryEntry[]>(
            "SELECT * FROM daily_stats ORDER BY day DESC LIMIT 30"
          );
          setHistory(result);
        } catch (error) {
          console.error("Failed to load history:", error);
        }
      }
      loadHistory();
    }
  }, [tab]);

  return (
    <main className="container">
      <header>
        <h1>Gnome Data Stats</h1>
        <nav>
          <button className={tab === "live" ? "active" : ""} onClick={() => setTab("live")}>Live</button>
          <button className={tab === "history" ? "active" : ""} onClick={() => setTab("history")}>History</button>
        </nav>
      </header>

      {tab === "live" ? (
        <>
          <div className="section">
            <label>Select Interface:</label>
            <select value={selected} onChange={(e) => handleInterfaceChange(e.target.value)}>
              {interfaces.map((iface) => (
                <option key={iface.name} value={iface.name}>
                  {iface.name} {iface.is_up ? "(Up)" : "(Down)"}
                </option>
              ))}
            </select>
          </div>

          <div className="stats-card">
            <h2>Monitoring: {selected || "None"}</h2>
            <div className="speed-row">
              <div className="speed-item">
                <span className="label">Download</span>
                <span className="value">{formatSpeed(speeds.download)}</span>
              </div>
              <div className="speed-item">
                <span className="label">Upload</span>
                <span className="value">{formatSpeed(speeds.upload)}</span>
              </div>
            </div>
          </div>
        </>
      ) : (
        <div className="history-list">
          {history.length === 0 ? (
            <p>No historical data yet.</p>
          ) : (
            history.map((entry, i) => (
              <div key={i} className="history-item">
                <div className="history-date">{entry.day} ({entry.interface})</div>
                <div className="history-values">
                  <span>⬇ {formatBytes(entry.download)}</span>
                  <span>⬆ {formatBytes(entry.upload)}</span>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </main>
  );
}

export default App;
