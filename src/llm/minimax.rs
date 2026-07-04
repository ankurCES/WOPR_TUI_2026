use futures::stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::types::*;

pub struct MinimaxProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<UsageInfo>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Deserialize)]
struct UsageInfo {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

impl MinimaxProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "abab6.5s-chat".into()),
        }
    }

    pub async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage { role: "system".into(), content: request.system_prompt.clone() },
                ChatMessage {
                    role: "user".into(),
                    content: format!("{}\n\nCONTEXT:\n{}", request.user_prompt, request.context_json),
                },
            ],
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let resp = self
            .client
            .post("https://api.minimax.chat/v1/text/chatcompletion_v2")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Server(e.to_string()))?;

        let status = resp.status().as_u16();
        match status {
            200 => {}
            401 => return Err(LlmError::Auth),
            429 => return Err(LlmError::RateLimited { retry_after_secs: 30 }),
            _ => {
                let text = resp.text().await.unwrap_or_default();
                return Err(LlmError::Server(format!("{status}: {text}")));
            }
        }

        let chat_resp: ChatResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::Parse(e.to_string()))?;

        let content = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let (pt, ct) = chat_resp
            .usage
            .map(|u| (u.prompt_tokens.unwrap_or(0), u.completion_tokens.unwrap_or(0)))
            .unwrap_or((0, 0));

        Ok(LlmResponse {
            content,
            prompt_tokens: pt,
            completion_tokens: ct,
            model: chat_resp.model.unwrap_or_else(|| self.model.clone()),
        })
    }

    pub fn generate_stream(&self, _request: &LlmRequest) -> StreamResult {
        Box::pin(stream::empty())
    }
}
