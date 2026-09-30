use native_host::{handle_message, read_message, write_message};
use std::io;

fn main() {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = stdout.lock();
    while let Ok(Some(message)) = read_message(&mut input) {
        let response = handle_message(&message);
        let Ok(payload) = serde_json::to_vec(&response) else {
            break;
        };
        if write_message(&mut output, &payload).is_err() {
            break;
        }
    }
}
