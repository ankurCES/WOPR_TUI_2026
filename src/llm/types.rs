use std::pin::Pin;

use futures::Stream;

pub struct LlmRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub context_json: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub model: String,
}

pub type StreamResult = Pin<Box<dyn Stream<Item = Result<String, LlmError>> + Send>>;

#[derive(Debug)]
pub enum LlmError {
    Auth,
    RateLimited { retry_after_secs: u32 },
    Server(String),
    Parse(String),
    BudgetExhausted,
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auth => write!(f, "authentication failed"),
            Self::RateLimited { retry_after_secs } => {
                write!(f, "rate limited, retry after {}s", retry_after_secs)
            }
            Self::Server(msg) => write!(f, "server error: {}", msg),
            Self::Parse(msg) => write!(f, "parse error: {}", msg),
            Self::BudgetExhausted => write!(f, "token budget exhausted"),
        }
    }
}

impl std::error::Error for LlmError {}
