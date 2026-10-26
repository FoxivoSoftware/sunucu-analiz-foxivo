use std::fs::{self, OpenOptions};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::Path;
use std::time::Duration;

/// A running server holds world/session.lock. Something answering on the
/// server's port only counts when this server's log is being written too,
/// otherwise it could be a different server on the same machine.
pub fn is_running(root: &Path) -> bool {
    lock_held(root) || (port_busy(root) && log_is_fresh(root))
}

fn log_is_fresh(root: &Path) -> bool {
    fs::metadata(root.join("logs").join("latest.log"))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age.as_secs() < 600)
}

fn lock_held(root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path().join("session.lock"))
        .filter(|p| p.exists())
        .any(
            |lock| match OpenOptions::new().read(true).write(true).open(&lock) {
                Ok(file) => match file.try_lock() {
                    Ok(()) => {
                        let _ = file.unlock();
                        false
                    }
                    Err(_) => true,
                },
                // Windows refuses to even open a file the server has locked
                Err(_) => true,
            },
        )
}

// Connecting instead of binding: opening a listening socket makes Windows
// show a firewall prompt for the app, an outgoing localhost connection doesn't.
fn port_busy(root: &Path) -> bool {
    let port = fs::read_to_string(root.join("server.properties"))
        .ok()
        .and_then(|t| crate::optimize::property(&t, "server-port"))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(25565);
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}
