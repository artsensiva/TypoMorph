//! Client for the paid, network-based prompt improvement path. The free,
//! offline path lives entirely in `core_engine::prompt_improver` and has no
//! dependency on this crate.
//!
//! Two backends:
//! - `BringYourOwnKey`: calls the Anthropic API directly with a user-supplied
//!   key. Free — the user pays Anthropic directly, not TypoMorph.
//! - `Managed`: calls TypoMorph's own hosted proxy, gated on an active Pro
//!   license. That proxy is a separate deployment this crate does not host;
//!   `DEFAULT_MANAGED_ENDPOINT` is a placeholder until it exists, and the
//!   request carries no auth token yet — a real deployment will need to
//!   attach a device/session credential here once the managed backend
//!   defines its auth scheme.

use core_engine::prompt_improver::improve_rule_based;
use thiserror::Error;

pub const ANTHROPIC_ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";
pub const ANTHROPIC_MODEL: &str = "claude-opus-5";
pub const DEFAULT_MANAGED_ENDPOINT: &str = "https://cloud.typomorph.com/v1/improve-prompt";

const SYSTEM_PROMPT: &str = "You improve a single user-written LLM prompt. Rewrite it for \
clarity and specificity while preserving the original intent, language, and meaning. Reply with \
only the improved prompt text: no explanation, no quotes, no markdown.";

#[derive(Debug, Error)]
pub enum CloudError {
    #[error("no Anthropic API key configured")]
    MissingApiKey,
    #[error("no active Pro license")]
    NotEntitled,
    #[error("cloud request failed: {0}")]
    Network(String),
    #[error("cloud response was malformed: {0}")]
    MalformedResponse(String),
}

pub enum Backend<'a> {
    BringYourOwnKey { api_key: &'a str },
    Managed,
}

pub trait CloudTransport {
    fn post_json(
        &self,
        endpoint: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Result<String, CloudError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UreqTransport;

impl CloudTransport for UreqTransport {
    fn post_json(
        &self,
        endpoint: &str,
        headers: &[(&str, &str)],
        body: &str,
    ) -> Result<String, CloudError> {
        let mut request = ureq::post(endpoint);
        for (name, value) in headers {
            request = request.set(name, value);
        }
        request
            .send_string(body)
            .map_err(|error| CloudError::Network(error.to_string()))?
            .into_string()
            .map_err(|error| CloudError::Network(error.to_string()))
    }
}

pub struct PromptCloudClient<T = UreqTransport> {
    transport: T,
    anthropic_endpoint: String,
    managed_endpoint: String,
}

impl PromptCloudClient<UreqTransport> {
    pub fn new() -> Self {
        Self {
            transport: UreqTransport,
            anthropic_endpoint: ANTHROPIC_ENDPOINT.to_string(),
            managed_endpoint: DEFAULT_MANAGED_ENDPOINT.to_string(),
        }
    }
}

impl Default for PromptCloudClient<UreqTransport> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: CloudTransport> PromptCloudClient<T> {
    pub fn with_transport(transport: T) -> Self {
        Self {
            transport,
            anthropic_endpoint: ANTHROPIC_ENDPOINT.to_string(),
            managed_endpoint: DEFAULT_MANAGED_ENDPOINT.to_string(),
        }
    }

    pub fn with_endpoints(
        transport: T,
        anthropic_endpoint: impl Into<String>,
        managed_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            transport,
            anthropic_endpoint: anthropic_endpoint.into(),
            managed_endpoint: managed_endpoint.into(),
        }
    }

    /// `is_pro` gates the `Managed` backend only; `BringYourOwnKey` never
    /// requires a TypoMorph license since the user pays Anthropic directly.
    pub fn improve(
        &self,
        text: &str,
        backend: Backend<'_>,
        is_pro: bool,
    ) -> Result<String, CloudError> {
        match backend {
            Backend::BringYourOwnKey { api_key } => {
                if api_key.trim().is_empty() {
                    return Err(CloudError::MissingApiKey);
                }
                self.call_anthropic(text, api_key)
            }
            Backend::Managed => {
                if !is_pro {
                    return Err(CloudError::NotEntitled);
                }
                self.call_managed(text)
            }
        }
    }

    /// Same as `improve`, but silently falls back to the free, offline
    /// rule-based improver on any cloud error (missing key, no entitlement,
    /// network failure, malformed response).
    pub fn improve_or_fallback(&self, text: &str, backend: Backend<'_>, is_pro: bool) -> String {
        self.improve(text, backend, is_pro)
            .unwrap_or_else(|_| improve_rule_based(text))
    }

    fn call_anthropic(&self, text: &str, api_key: &str) -> Result<String, CloudError> {
        let body = serde_json::json!({
            "model": ANTHROPIC_MODEL,
            "max_tokens": 4096,
            "system": SYSTEM_PROMPT,
            "output_config": { "effort": "low" },
            "messages": [{ "role": "user", "content": text }],
        })
        .to_string();

        let raw = self.transport.post_json(
            &self.anthropic_endpoint,
            &[
                ("Content-Type", "application/json"),
                ("x-api-key", api_key),
                ("anthropic-version", ANTHROPIC_VERSION),
            ],
            &body,
        )?;

        parse_anthropic_response(&raw)
    }

    fn call_managed(&self, text: &str) -> Result<String, CloudError> {
        let body = serde_json::json!({ "text": text }).to_string();
        let raw = self.transport.post_json(
            &self.managed_endpoint,
            &[("Content-Type", "application/json")],
            &body,
        )?;

        let value: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|error| CloudError::MalformedResponse(error.to_string()))?;
        value
            .get("improved")
            .and_then(|field| field.as_str())
            .map(|text| text.trim().to_string())
            .ok_or_else(|| CloudError::MalformedResponse("missing 'improved' field".to_string()))
    }
}

fn parse_anthropic_response(raw: &str) -> Result<String, CloudError> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|error| CloudError::MalformedResponse(error.to_string()))?;

    if let Some(message) = value
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(|message| message.as_str())
    {
        return Err(CloudError::Network(message.to_string()));
    }

    value
        .get("content")
        .and_then(|content| content.as_array())
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|block| block.get("type").and_then(|t| t.as_str()) == Some("text"))
        })
        .and_then(|block| block.get("text"))
        .and_then(|text| text.as_str())
        .map(|text| text.trim().to_string())
        .ok_or_else(|| {
            CloudError::MalformedResponse("no text content block in response".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTransport {
        response: String,
    }

    impl CloudTransport for MockTransport {
        fn post_json(
            &self,
            _endpoint: &str,
            _headers: &[(&str, &str)],
            _body: &str,
        ) -> Result<String, CloudError> {
            Ok(self.response.clone())
        }
    }

    #[test]
    fn byok_parses_anthropic_text_block() {
        let client = PromptCloudClient::with_transport(MockTransport {
            response: serde_json::json!({
                "content": [{"type": "text", "text": "Improved prompt text."}]
            })
            .to_string(),
        });
        let result = client
            .improve(
                "original",
                Backend::BringYourOwnKey { api_key: "sk-test" },
                false,
            )
            .unwrap();
        assert_eq!(result, "Improved prompt text.");
    }

    #[test]
    fn byok_requires_a_non_empty_key() {
        let client = PromptCloudClient::with_transport(MockTransport {
            response: String::new(),
        });
        let error = client
            .improve("original", Backend::BringYourOwnKey { api_key: "" }, false)
            .unwrap_err();
        assert!(matches!(error, CloudError::MissingApiKey));
    }

    #[test]
    fn managed_backend_requires_pro_entitlement() {
        let client = PromptCloudClient::with_transport(MockTransport {
            response: String::new(),
        });
        let error = client
            .improve("original", Backend::Managed, false)
            .unwrap_err();
        assert!(matches!(error, CloudError::NotEntitled));
    }

    #[test]
    fn managed_backend_parses_improved_field_when_entitled() {
        let client = PromptCloudClient::with_transport(MockTransport {
            response: serde_json::json!({ "improved": "Better prompt." }).to_string(),
        });
        let result = client.improve("original", Backend::Managed, true).unwrap();
        assert_eq!(result, "Better prompt.");
    }

    #[test]
    fn anthropic_error_response_surfaces_as_network_error() {
        let client = PromptCloudClient::with_transport(MockTransport {
            response: serde_json::json!({ "error": { "message": "invalid x-api-key" } })
                .to_string(),
        });
        let error = client
            .improve(
                "original",
                Backend::BringYourOwnKey { api_key: "bad" },
                false,
            )
            .unwrap_err();
        assert!(matches!(error, CloudError::Network(_)));
    }

    #[test]
    fn falls_back_to_local_rule_based_improver_on_any_cloud_error() {
        let client = PromptCloudClient::with_transport(MockTransport {
            response: String::new(),
        });
        let improved = client.improve_or_fallback(
            "pls fix this",
            Backend::BringYourOwnKey { api_key: "" },
            false,
        );
        assert_eq!(improved, improve_rule_based("pls fix this"));
    }
}
