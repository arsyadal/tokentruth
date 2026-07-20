use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    ToolResult,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnUsage {
    pub turn_id: String,
    pub role: Role,
    pub timestamp: DateTime<Utc>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: Option<u64>,
    pub model: Option<String>,
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub project_path: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub turns: Vec<TurnUsage>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TokenBreakdown {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
}

impl SessionRecord {
    pub fn breakdown(&self) -> TokenBreakdown {
        let mut b = TokenBreakdown::default();
        for t in &self.turns {
            b.input_tokens += t.input_tokens;
            b.output_tokens += t.output_tokens;
            b.cache_read_tokens += t.cache_read_tokens;
            b.cache_write_tokens += t.cache_write_tokens;
            b.reasoning_tokens += t.reasoning_tokens.unwrap_or(0);
        }
        b
    }

    pub fn primary_model(&self) -> Option<&str> {
        self.turns.iter().find_map(|t| t.model.as_deref())
    }
}

impl TokenBreakdown {
    pub fn total(&self) -> u64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_read_tokens
            + self.cache_write_tokens
            + self.reasoning_tokens
    }
}
