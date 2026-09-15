use std::io;

use licensing::LicenseStore;
use native_host::{handle, read_message, write_message, Request, Response};

fn main() {
    let is_pro = LicenseStore::default_path()
        .ok()
        .map(LicenseStore::new)
        .and_then(|store| store.load().ok())
        .flatten()
        .is_some();

    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();

    loop {
        let message = match read_message(&mut stdin_lock) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => break,
            Err(_) => break,
        };

        let response = match serde_json::from_slice::<Request>(&message) {
            Ok(request) => handle(request, is_pro),
            Err(error) => Response::Error {
                ok: false,
                error: format!("invalid request: {error}"),
            },
        };

        let payload = serde_json::to_vec(&response).unwrap_or_else(|_| b"{\"ok\":false}".to_vec());
        if write_message(&mut stdout_lock, &payload).is_err() {
            break;
        }
    }
}
