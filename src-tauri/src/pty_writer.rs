use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A session that c9watch has taken over.
pub struct ManagedSession {
    pub session_id: String,
    pub project_path: String,
}

pub type SessionMap = Arc<Mutex<HashMap<String, ManagedSession>>>;

pub fn new_session_map() -> SessionMap {
    Arc::new(Mutex::new(HashMap::new()))
}

/// Mark the session as managed by c9watch, optionally killing the existing process.
/// If pid is 0 the kill step is skipped (useful for already-ended sessions).
/// After registering, the caller should also tell the bridge to resume the session.
pub fn take_over_session(
    map: &SessionMap,
    pid: u32,
    session_id: &str,
    project_path: &str,
) -> Result<(), String> {
    // TODO: re-enable kill once bridge is stable
    // if pid > 0 {
    //     crate::actions::stop_session(pid)?;
    //     std::thread::sleep(std::time::Duration::from_millis(500));
    // }
    eprintln!(
        "[pty_writer] Took over session {} (pid={}, kill skipped for testing)",
        session_id, pid
    );

    touch_session_file(session_id);

    let mut sessions = map
        .lock()
        .map_err(|e| format!("Lock poisoned: {}", e))?;
    sessions.insert(
        session_id.to_string(),
        ManagedSession {
            session_id: session_id.to_string(),
            project_path: project_path.to_string(),
        },
    );

    Ok(())
}

/// Get the project path for a managed session.
pub fn get_project_path(map: &SessionMap, session_id: &str) -> Result<String, String> {
    let sessions = map
        .lock()
        .map_err(|e| format!("Lock poisoned: {}", e))?;
    sessions
        .get(session_id)
        .map(|m| m.project_path.clone())
        .ok_or_else(|| format!("Session {} is not managed", session_id))
}

/// Check if a session is currently managed by c9watch.
pub fn is_managed(map: &SessionMap, session_id: &str) -> bool {
    map.lock()
        .map(|sessions| sessions.contains_key(session_id))
        .unwrap_or(false)
}

/// Info about a managed session, for the polling system.
pub struct ManagedSessionInfo {
    pub session_id: String,
    pub pid: u32,
}

/// Get all currently managed sessions.
pub fn get_managed_sessions(map: &SessionMap) -> Vec<ManagedSessionInfo> {
    map.lock()
        .map(|sessions| {
            sessions
                .values()
                .map(|m| ManagedSessionInfo {
                    session_id: m.session_id.clone(),
                    pid: 0, // No persistent process
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Touch (update mtime of) the session JSONL file so polling re-detects the session.
pub fn touch_session_file(session_id: &str) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    let projects_dir = home.join(".claude").join("projects");
    let entries = match std::fs::read_dir(&projects_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let session_file = entry.path().join(format!("{}.jsonl", session_id));
        if session_file.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::io::AsRawFd;
                if let Ok(file) = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&session_file)
                {
                    let times = [
                        libc::timespec {
                            tv_sec: 0,
                            tv_nsec: libc::UTIME_NOW,
                        },
                        libc::timespec {
                            tv_sec: 0,
                            tv_nsec: libc::UTIME_NOW,
                        },
                    ];
                    unsafe {
                        libc::futimens(file.as_raw_fd(), times.as_ptr());
                    }
                }
            }
            eprintln!("[pty_writer] Touched session file: {:?}", session_file);
            return;
        }
    }
    eprintln!(
        "[pty_writer] Warning: could not find session file for {}",
        session_id
    );
}
