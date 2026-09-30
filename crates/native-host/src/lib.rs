//! Content-free Native Messaging readiness endpoint.
//!
//! Correction remains unavailable until an authenticated desktop lifecycle and
//! field-owning browser adapter exist. Legacy prompt actions are not supported.
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

/// Deliberately below browser protocol maxima: this endpoint accepts no text.
pub const MAX_FRAME_BYTES: usize = 4096;

#[derive(Deserialize)]
struct Envelope {
    action: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Response {
    pub ok: bool,
    pub protocol_version: u32,
    pub correction_available: bool,
    pub code: &'static str,
}

/// Never format deserialization errors: they may contain submitted values.
pub fn handle_message(bytes: &[u8]) -> Response {
    let code = if bytes.len() > MAX_FRAME_BYTES {
        "request_too_large"
    } else {
        match serde_json::from_slice::<Envelope>(bytes) {
            Ok(request) if request.action == "status" => "desktop_connection_required",
            Ok(request) if request.action == "correct_layout" => "correction_unavailable",
            Ok(_) => "unsupported_action",
            Err(_) => "invalid_request",
        }
    };
    Response {
        ok: code == "desktop_connection_required",
        protocol_version: 1,
        correction_available: false,
        code,
    }
}

pub fn read_message<R: Read>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut length_buf = [0; 4];
    // Only zero bytes at a frame boundary are clean EOF; a partial prefix is an error.
    loop {
        match reader.read(&mut length_buf[..1]) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    reader.read_exact(&mut length_buf[1..])?;
    let length = u32::from_ne_bytes(length_buf) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid message length",
        ));
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    Ok(Some(bytes))
}

pub fn write_message<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid message length",
        ));
    }
    writer.write_all(&(payload.len() as u32).to_ne_bytes())?;
    writer.write_all(payload)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concatenated_frames_and_clean_eof() {
        let mut data = Vec::new();
        write_message(&mut data, b"one").unwrap();
        write_message(&mut data, b"two").unwrap();
        let mut input = io::Cursor::new(data);
        assert_eq!(read_message(&mut input).unwrap().unwrap(), b"one");
        assert_eq!(read_message(&mut input).unwrap().unwrap(), b"two");
        assert!(read_message(&mut input).unwrap().is_none());
    }

    #[test]
    fn truncated_prefix_and_payload_are_errors() {
        for count in 1..4 {
            assert_eq!(
                read_message(&mut io::Cursor::new(vec![0; count]))
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::UnexpectedEof
            );
        }
        let mut data = 10u32.to_ne_bytes().to_vec();
        data.extend_from_slice(b"short");
        assert_eq!(
            read_message(&mut io::Cursor::new(data)).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn excessive_length_is_refused_before_payload_read_or_allocation() {
        for length in [0, MAX_FRAME_BYTES as u32 + 1, u32::MAX] {
            let mut reader = io::Cursor::new(length.to_ne_bytes());
            assert_eq!(
                read_message(&mut reader).unwrap_err().kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(reader.position(), 4);
        }
        let mut output = Vec::new();
        assert!(write_message(&mut output, &vec![0; MAX_FRAME_BYTES + 1]).is_err());
        assert!(output.is_empty());
    }

    #[test]
    fn readiness_never_authorizes_correction() {
        for action in ["status", "correct_layout"] {
            let response =
                handle_message(format!(r#"{{"action":"{action}","text":"SECRET"}}"#).as_bytes());
            assert!(!response.correction_available);
            assert_eq!(response.ok, action == "status");
            assert!(!serde_json::to_string(&response).unwrap().contains("SECRET"));
        }
    }

    #[test]
    fn malformed_and_removed_actions_cannot_echo_input_or_credentials() {
        for request in [
            br#"{"action":"improve_prompt","text":"SECRET","cloud":true,"api_key":"KEY"}"#
                .as_slice(),
            br#"{"action":{"SECRET":"KEY"}}"#,
            br#"{"action":"SECRET"}"#,
            br#"{"action":"status","action":"SECRET"}"#,
            b"SECRET invalid json",
        ] {
            let response = handle_message(request);
            assert!(!response.ok);
            let output = serde_json::to_string(&response).unwrap();
            assert!(!output.contains("SECRET") && !output.contains("KEY"));
        }
    }
}
