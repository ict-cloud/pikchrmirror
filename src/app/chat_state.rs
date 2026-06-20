#[cfg(feature = "llm")]
use std::sync::Arc;

#[cfg(feature = "llm")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

#[cfg(feature = "llm")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelStatus {
    Unloaded,
    Loading,
    Ready,
    Error(String),
}

#[cfg(feature = "llm")]
impl std::fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelStatus::Unloaded => write!(f, "Model not loaded"),
            ModelStatus::Loading => write!(f, "Loading model…"),
            ModelStatus::Ready => write!(f, "Ready"),
            ModelStatus::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

#[cfg(feature = "llm")]
#[derive(Debug, Clone)]
pub struct ChatTurn {
    pub role: Role,
    pub text: String,
}

#[cfg(feature = "llm")]
#[derive(Clone)]
pub struct ChatState {
    pub visible: bool,
    pub input: String,
    pub messages: Vec<ChatTurn>,
    pub model: Option<Arc<crate::llm::MistralRs>>,
    pub status: ModelStatus,
    pub generating: bool,
    pub proposed_code: Option<String>,
}

#[cfg(feature = "llm")]
impl Default for ChatState {
    fn default() -> Self {
        Self {
            visible: false,
            input: String::new(),
            messages: Vec::new(),
            model: None,
            status: ModelStatus::Unloaded,
            generating: false,
            proposed_code: None,
        }
    }
}

#[cfg(feature = "llm")]
impl std::fmt::Debug for ChatState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatState")
            .field("visible", &self.visible)
            .field("input", &self.input)
            .field("messages", &self.messages)
            .field("model", &self.model.is_some())
            .field("status", &self.status)
            .field("generating", &self.generating)
            .field("proposed_code", &self.proposed_code)
            .finish()
    }
}
