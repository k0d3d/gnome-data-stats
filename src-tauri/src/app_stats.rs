use std::collections::HashMap;
use serde::Serialize;
use procfs::process::Process;
use std::fs;

#[derive(Serialize, Clone, Debug, Default)]
pub struct AppUsage {
    pub name: String,
    pub download: u64,
    pub upload: u64,
}

pub fn get_process_map() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    if let Ok(all_procs) = procfs::process::all_processes() {
        for p in all_procs {
            if let Ok(proc) = p {
                let pid = proc.pid() as u32;
                let name = proc.stat().map(|s| s.comm).unwrap_or_else(|_| "unknown".to_string());
                map.insert(pid, name);
            }
        }
    }
    map
}

// In a real scenario, this would involve complex packet capture matching inodes.
// For this prototype, we'll use a simpler heuristic: reading /proc/[pid]/net/dev if available 
// or /proc/[pid]/io for general I/O as a proxy for network activity if we can't get root packet capture easily.
// However, the user asked for root auth dialog, so we will implement a skeleton for the privileged collector.
