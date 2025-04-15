use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AiApi {
    pub key: String,
    pub url: String,
    pub model: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub record_device: Option<String>,
    pub ollama_url: String,
    pub ollama_port: u16,
    pub ollama_model: String,

    pub font_size: f32,
    pub w: f32,
    pub h: f32,

    pub gpt: Option<AiApi>,
    pub deepseek: Option<AiApi>,
    pub grok: Option<AiApi>,

    pub eleven: Option<AiApi>,

    pub whisper_model: String,
    pub chat_msg_wait: u64,
}


