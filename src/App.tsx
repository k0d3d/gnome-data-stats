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
  session_download: number;
  session_upload: number;
}

interface HistoryEntry {
  period: string;
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
  const [tab, setTab] = useState<"live" | "history" | "settings">("live");
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [historyPeriod, setHistoryPeriod] = useState<"hourly" | "daily" | "monthly">("daily");
  const [limit, setLimit] = useState<number>(() => {
    return Number(localStorage.getItem("data-limit")) || 100; // GB
  });

  const [speeds, setSpeeds] = useState<{
    download: number;
    upload: number;
    session_dl: number;
    session_ul: number;
    daily_dl: number;
    daily_ul: number;
  }>({
    download: 0,
    upload: 0,
    session_dl: 0,
    session_ul: 0,
    daily_dl: 0,
    daily_ul: 0,
  });

  useEffect(() => {
    async function fetchInterfaces() {
      try {
        const result = await invoke<NetworkInterface[]>("get_network_interfaces");
        setInterfaces(result);
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

  const loadDailyTotal = async (iface: string) => {
    try {
      const db = await Database.load("sqlite:stats.db");
      const day = new Date().toISOString().split("T")[0];
      const result = await db.select<any[]>(
        "SELECT * FROM daily_stats WHERE day = ? AND interface = ?",
        [day, iface]
      );
      if (result.length > 0) {
        setSpeeds(s => ({
          ...s,
          daily_dl: result[0].download,
          daily_ul: result[0].upload
        }));
      }
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    if (selected) loadDailyTotal(selected);
  }, [selected]);

  useEffect(() => {
    const unlisten = listen<SpeedStats>("network-speed", (event) => {
      if (event.payload.interface === selected) {
        setSpeeds(prev => ({
          ...prev,
          download: event.payload.download_speed,
          upload: event.payload.upload_speed,
          session_dl: event.payload.session_download,
          session_ul: event.payload.session_upload,
        }));
      }
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [selected]);

  const loadHistory = async () => {
    try {
      const result = await invoke<HistoryEntry[]>("get_history", { 
        periodType: historyPeriod,
        interface: selected
      });
      setHistory(result);
    } catch (error) {
      console.error("Failed to load history:", error);
    }
  };

  useEffect(() => {
    if (tab === "history") {
      loadHistory();
    }
  }, [tab, historyPeriod, selected]);

  useEffect(() => {
    const unlistenSaved = listen("stats-saved", () => {
      if (selected) loadDailyTotal(selected);
      if (tab === "history") loadHistory();
    });

    return () => {
      unlistenSaved.then((f) => f());
    };
  }, [tab, selected, historyPeriod]);

  const handleInterfaceChange = (name: string) => {
    setSelected(name);
    localStorage.setItem("last-interface", name);
    setSpeeds({ download: 0, upload: 0, session_dl: 0, session_ul: 0, daily_dl: 0, daily_ul: 0 });
  };

  const totalMonthly = (speeds.daily_dl + speeds.daily_ul);
  const limitBytes = limit * 1024 * 1024 * 1024;
  const percentage = Math.min(100, (totalMonthly / limitBytes) * 100);

  return (
    <main className="container">
      <header>
        <h1>Gnome Data Stats</h1>
        <nav>
          <button className={tab === "live" ? "active" : ""} onClick={() => setTab("live")}>Live</button>
          <button className={tab === "history" ? "active" : ""} onClick={() => setTab("history")}>History</button>
          <button className={tab === "settings" ? "active" : ""} onClick={() => setTab("settings")}>Plan</button>
        </nav>
      </header>

      {tab === "live" && (
        <div className="fade-in">
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
            <div className="card-header">
              <h2>Real-time Speeds</h2>
              <span className="badge">LIVE</span>
            </div>
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

          <div className="usage-grid">
            <div className="usage-card small">
              <span className="label">Session Total</span>
              <span className="large-value">{formatBytes(speeds.session_dl + speeds.session_ul)}</span>
              <div className="sub-values">
                <span>⬇ {formatBytes(speeds.session_dl)}</span>
                <span>⬆ {formatBytes(speeds.session_ul)}</span>
              </div>
            </div>
            <div className="usage-card small">
              <span className="label">Today's Total</span>
              <span className="large-value">{formatBytes(speeds.daily_dl + speeds.daily_ul)}</span>
              <div className="sub-values">
                <span>⬇ {formatBytes(speeds.daily_dl)}</span>
                <span>⬆ {formatBytes(speeds.daily_ul)}</span>
              </div>
            </div>
          </div>

          <div className="plan-card">
            <div className="plan-info">
              <span className="label">Data Plan Usage (Daily)</span>
              <span className="percentage">{percentage.toFixed(1)}%</span>
            </div>
            <div className="progress-bar">
              <div className="progress-fill" style={{ width: `${percentage}%` }}></div>
            </div>
            <p className="hint">Plan: {limit} GB / Month (Heuristic: Showing Daily against Plan)</p>
          </div>
        </div>
      )}

      {tab === "history" && (
        <div className="fade-in">
          <div className="section" style={{ marginBottom: "1rem" }}>
            <label>Interface:</label>
            <select value={selected} onChange={(e) => handleInterfaceChange(e.target.value)}>
              {interfaces.map((iface) => (
                <option key={iface.name} value={iface.name}>
                  {iface.name}
                </option>
              ))}
            </select>
          </div>
          
          <div className="period-selector">
            <button 
              className={historyPeriod === "hourly" ? "active" : ""} 
              onClick={() => setHistoryPeriod("hourly")}
            >
              Hourly
            </button>
            <button 
              className={historyPeriod === "daily" ? "active" : ""} 
              onClick={() => setHistoryPeriod("daily")}
            >
              Daily
            </button>
            <button 
              className={historyPeriod === "monthly" ? "active" : ""} 
              onClick={() => setHistoryPeriod("monthly")}
            >
              Monthly
            </button>
          </div>
          
          <div className="history-list">
            {history.length === 0 ? (
              <p className="empty">No historical data yet.</p>
            ) : (
              history.map((entry, i) => (
                <div key={i} className="history-item">
                  <div className="history-info">
                    <div className="history-date">{entry.period}</div>
                    <div className="history-iface">{entry.interface}</div>
                  </div>
                  <div className="history-values">
                    <span>⬇ {formatBytes(entry.download)}</span>
                    <span>⬆ {formatBytes(entry.upload)}</span>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {tab === "settings" && (
        <div className="settings-panel fade-in">
          <div className="section">
            <label>Monthly Data Limit (GB)</label>
            <input 
              type="number" 
              value={limit} 
              onChange={(e) => {
                const val = Number(e.target.value);
                setLimit(val);
                localStorage.setItem("data-limit", val.toString());
              }} 
            />
          </div>
          <div className="info-box">
            <h3>Traffic Classification</h3>
            <p><strong>Total Traffic:</strong> Includes all bytes moving through the interface (LAN + Internet).</p>
            <p><strong>Note:</strong> Perfectly splitting Internet vs LAN requires root-level packet inspection, which this app avoids for security. Use "Plan" to monitor your ISP limits.</p>
          </div>
        </div>
      )}
    </main>
  );
}

export default App;
