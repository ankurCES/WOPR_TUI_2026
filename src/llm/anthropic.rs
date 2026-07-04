use futures::stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::types::*;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    system: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
    model: String,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "claude-sonnet-4-6".into()),
        }
    }

    pub async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let body = ApiRequest {
            model: self.model.clone(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            system: request.system_prompt.clone(),
            messages: vec![Message {
                role: "user",
                content: format!("{}\n\nCONTEXT:\n{}", request.user_prompt, request.context_json),
            }],
            stream: false,
        };

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Server(e.to_string()))?;

        let status = resp.status().as_u16();
        match status {
            200 => {}
            401 => return Err(LlmError::Auth),
            429 => {
                let retry = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30);
                return Err(LlmError::RateLimited { retry_after_secs: retry });
            }
            _ => {
                let text = resp.text().await.unwrap_or_default();
                return Err(LlmError::Server(format!("{status}: {text}")));
            }
        }

        let api_resp: ApiResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;

        let content = api_resp
            .content
            .into_iter()
            .map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(LlmResponse {
            content,
            prompt_tokens: api_resp.usage.input_tokens,
            completion_tokens: api_resp.usage.output_tokens,
            model: api_resp.model,
        })
    }

    // ponytail: full SSE streaming deferred — return chunked non-stream for now
    pub fn generate_stream(&self, _request: &LlmRequest) -> StreamResult {
        Box::pin(stream::empty())
    }
}
