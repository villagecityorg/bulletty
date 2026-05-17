/// LLM integration module — DeepSeek API client, Smart Pick, Daily Digest.
pub mod client;
pub mod digest;
pub mod smart_pick;

use serde::{Deserialize, Serialize};

use crate::core::config::LlmConfig;

/// Runtime configuration for the LLM client.
pub struct LlmRuntime {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
}

impl LlmRuntime {
    pub fn from_config(cfg: &LlmConfig) -> Option<Self> {
        let api_key = cfg.api_key.clone()?;
        Some(Self {
            api_url: cfg
                .api_url
                .clone()
                .unwrap_or_else(|| "https://api.deepseek.com".to_string()),
            api_key,
            model: cfg.model.clone().unwrap_or_else(|| "deepseek-chat".to_string()),
        })
    }
}

/// A single message in the chat conversation.
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Request body for /v1/chat/completions.
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// Response from /v1/chat/completions.
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponseMessage {
    pub content: String,
}

/// Call the DeepSeek chat API and return the assistant's response text.
pub fn chat_completion(
    runtime: &LlmRuntime,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}/v1/chat/completions", runtime.api_url.trim_end_matches('/'));

    let body = ChatRequest {
        model: runtime.model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(2048),
    };

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", runtime.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("DeepSeek API error ({}): {}", status, text).into());
    }

    let chat_resp: ChatResponse = resp.json()?;
    let content = chat_resp
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    Ok(content)
}
