//! Chrome/Firefox Native Messaging host logic for the TypoMorph browser
//! extensions. This is a thin, stateless request/response layer over the
//! same libraries the daemon uses (`core-engine`, `prompt-cloud`) — it does
//! not talk to the running `typomorph` daemon process; native messaging
//! spawns a fresh subprocess per browser connection, so there is nothing
//! long-lived to connect to.
//!
//! Wire format: Chrome/Firefox Native Messaging framing — a 4-byte
//! native-endian length prefix followed by that many bytes of UTF-8 JSON,
//! in both directions.

use std::io::{self, Read, Write};

use core_engine::layout::evaluate_layout_candidates;
use core_engine::prompt_detector::PromptDetector;
use core_engine::prompt_improver::improve_rule_based;
use core_engine::LanguageClassifier;
use prompt_cloud::{Backend, PromptCloudClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Request {
    Status,
    CorrectLayout {
        text: String,
        #[serde(default = "default_layout")]
        layout: String,
    },
    ImprovePrompt {
        text: String,
        #[serde(default)]
        cloud: bool,
        #[serde(default)]
        api_key: Option<String>,
    },
}

fn default_layout() -> String {
    "us".to_string()
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Response {
    Status {
        ok: bool,
        tier: String,
    },
    CorrectLayout {
        ok: bool,
        language: String,
        switched: bool,
        target_layout: Option<String>,
        corrected: String,
    },
    ImprovePrompt {
        ok: bool,
        mode: String,
        prompt_detected: bool,
        confidence: f64,
        improved: String,
    },
    Error {
        ok: bool,
        error: String,
    },
}

/// `is_pro` is read once from `licensing::LicenseStore` by the binary before
/// entering the request loop; this library never touches the filesystem or
/// network on its own, which keeps it fully unit-testable.
pub fn handle(request: Request, is_pro: bool) -> Response {
    match request {
        Request::Status => Response::Status {
            ok: true,
            tier: if is_pro { "pro" } else { "free" }.to_string(),
        },
        Request::CorrectLayout { text, layout } => correct_layout(&text, &layout),
        Request::ImprovePrompt {
            text,
            cloud,
            api_key,
        } => improve_prompt(&text, cloud, api_key, is_pro),
    }
}

fn correct_layout(text: &str, layout: &str) -> Response {
    let classifier = LanguageClassifier::new();
    let decision = evaluate_layout_candidates(text, layout, &classifier, 0.60);
    Response::CorrectLayout {
        ok: true,
        language: format!("{:?}", decision.language),
        switched: decision.switch,
        target_layout: decision.target_layout.map(str::to_string),
        corrected: decision.corrected,
    }
}

fn improve_prompt(text: &str, cloud: bool, api_key: Option<String>, is_pro: bool) -> Response {
    let signal = PromptDetector::new().detect(text);

    if !cloud {
        return Response::ImprovePrompt {
            ok: true,
            mode: "local".to_string(),
            prompt_detected: signal.is_prompt,
            confidence: signal.confidence,
            improved: improve_rule_based(text),
        };
    }

    let client = PromptCloudClient::new();
    let api_key = api_key.filter(|key| !key.trim().is_empty());

    let (backend, mode) = match &api_key {
        Some(api_key) => (
            Backend::BringYourOwnKey { api_key },
            "cloud-byok".to_string(),
        ),
        None if is_pro => (Backend::Managed, "cloud-managed".to_string()),
        None => {
            return Response::Error {
                ok: false,
                error: "cloud improvement requires an api_key or an active Pro license".to_string(),
            };
        }
    };

    match client.improve(text, backend, is_pro) {
        Ok(improved) => Response::ImprovePrompt {
            ok: true,
            mode,
            prompt_detected: signal.is_prompt,
            confidence: signal.confidence,
            improved,
        },
        Err(_) => Response::ImprovePrompt {
            ok: true,
            mode: "local-fallback".to_string(),
            prompt_detected: signal.is_prompt,
            confidence: signal.confidence,
            improved: improve_rule_based(text),
        },
    }
}

/// Reads one Native Messaging frame. `Ok(None)` means clean EOF (the browser
/// closed the port) — the caller should exit its loop, not treat it as an error.
pub fn read_message<R: Read>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut length_buf = [0u8; 4];
    match reader.read_exact(&mut length_buf) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }
    let length = u32::from_ne_bytes(length_buf) as usize;
    let mut buffer = vec![0u8; length];
    reader.read_exact(&mut buffer)?;
    Ok(Some(buffer))
}

pub fn write_message<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    let length = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    writer.write_all(&length.to_ne_bytes())?;
    writer.write_all(payload)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_framing_round_trips() {
        let mut buffer = Vec::new();
        write_message(&mut buffer, br#"{"ok":true}"#).unwrap();

        let mut cursor = io::Cursor::new(buffer);
        let message = read_message(&mut cursor).unwrap().unwrap();
        assert_eq!(message, br#"{"ok":true}"#);
    }

    #[test]
    fn read_message_returns_none_on_clean_eof() {
        let mut cursor = io::Cursor::new(Vec::<u8>::new());
        assert!(read_message(&mut cursor).unwrap().is_none());
    }

    #[test]
    fn status_reports_tier_from_injected_flag() {
        assert_eq!(
            handle(Request::Status, false),
            Response::Status {
                ok: true,
                tier: "free".to_string(),
            }
        );
        assert_eq!(
            handle(Request::Status, true),
            Response::Status {
                ok: true,
                tier: "pro".to_string(),
            }
        );
    }

    #[test]
    fn correct_layout_fixes_gibberish() {
        let response = handle(
            Request::CorrectLayout {
                text: "ghbdtn".to_string(),
                layout: "us".to_string(),
            },
            false,
        );
        assert_eq!(
            response,
            Response::CorrectLayout {
                ok: true,
                language: "Russian".to_string(),
                switched: true,
                target_layout: Some("ru".to_string()),
                corrected: "привет".to_string(),
            }
        );
    }

    #[test]
    fn improve_prompt_local_never_touches_network() {
        let response = handle(
            Request::ImprovePrompt {
                text: "pls write a poem".to_string(),
                cloud: false,
                api_key: None,
            },
            false,
        );
        assert_eq!(
            response,
            Response::ImprovePrompt {
                ok: true,
                mode: "local".to_string(),
                prompt_detected: true,
                confidence: 0.60,
                improved: improve_rule_based("pls write a poem"),
            }
        );
    }

    #[test]
    fn improve_prompt_cloud_without_key_or_pro_errors_out() {
        let response = handle(
            Request::ImprovePrompt {
                text: "write a poem".to_string(),
                cloud: true,
                api_key: None,
            },
            false,
        );
        assert!(matches!(response, Response::Error { ok: false, .. }));
    }
}
