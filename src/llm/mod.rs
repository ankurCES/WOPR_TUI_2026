pub mod anthropic;
pub mod budget;
pub mod cache;
pub mod minimax;
pub mod stub;
pub mod types;

use types::{LlmError, LlmRequest, LlmResponse, StreamResult};

pub trait LlmProvider: Send + Sync {
    fn generate(
        &self,
        request: &LlmRequest,
    ) -> impl std::future::Future<Output = Result<LlmResponse, LlmError>> + Send;

    fn generate_stream(&self, request: &LlmRequest) -> StreamResult;
}

pub fn create_provider(name: &str, api_key: Option<String>, model: Option<String>) -> Box<dyn LlmProviderBoxed> {
    match name {
        "anthropic" => {
            let key = api_key.expect("anthropic provider requires api_key in settings");
            Box::new(anthropic::AnthropicProvider::new(key, model))
        }
        "minimax" => {
            let key = api_key.expect("minimax provider requires api_key in settings");
            Box::new(minimax::MinimaxProvider::new(key, model))
        }
        _ => Box::new(stub::StubProvider::new()),
    }
}

// ponytail: object-safe wrapper since LlmProvider uses RPITIT
pub trait LlmProviderBoxed: Send + Sync {
    fn generate_boxed<'a>(
        &'a self,
        request: &'a LlmRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse, LlmError>> + Send + 'a>>;

    fn generate_stream_boxed(&self, request: &LlmRequest) -> StreamResult;
}

impl LlmProviderBoxed for stub::StubProvider {
    fn generate_boxed<'a>(
        &'a self,
        request: &'a LlmRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse, LlmError>> + Send + 'a>> {
        Box::pin(self.generate(request))
    }

    fn generate_stream_boxed(&self, request: &LlmRequest) -> StreamResult {
        self.generate_stream(request)
    }
}

impl LlmProviderBoxed for anthropic::AnthropicProvider {
    fn generate_boxed<'a>(
        &'a self,
        request: &'a LlmRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse, LlmError>> + Send + 'a>> {
        Box::pin(self.generate(request))
    }

    fn generate_stream_boxed(&self, request: &LlmRequest) -> StreamResult {
        self.generate_stream(request)
    }
}

impl LlmProviderBoxed for minimax::MinimaxProvider {
    fn generate_boxed<'a>(
        &'a self,
        request: &'a LlmRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LlmResponse, LlmError>> + Send + 'a>> {
        Box::pin(self.generate(request))
    }

    fn generate_stream_boxed(&self, request: &LlmRequest) -> StreamResult {
        self.generate_stream(request)
    }
}
