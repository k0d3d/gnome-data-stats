use serde::Serialize;
use std::collections::HashMap;

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
