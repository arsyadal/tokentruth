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

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(input: u64, output: u64, cache_read: u64, cache_write: u64, reasoning: Option<u64>, model: Option<&str>) -> TurnUsage {
        TurnUsage {
            turn_id: "t".into(),
            role: Role::Assistant,
            timestamp: Utc::now(),
            input_tokens: input,
            output_tokens: output,
            cache_read_tokens: cache_read,
            cache_write_tokens: cache_write,
            reasoning_tokens: reasoning,
            model: model.map(String::from),
            tool_name: None,
        }
    }

    #[test]
    fn breakdown_sums_across_turns() {
        let session = SessionRecord {
            session_id: "s".into(),
            project_path: "/p".into(),
            started_at: Utc::now(),
            ended_at: Utc::now(),
            turns: vec![
                turn(10, 20, 30, 40, Some(5), Some("claude-sonnet-5")),
                turn(1, 2, 3, 4, None, None),
            ],
        };
        let b = session.breakdown();
        assert_eq!(b.input_tokens, 11);
        assert_eq!(b.output_tokens, 22);
        assert_eq!(b.cache_read_tokens, 33);
        assert_eq!(b.cache_write_tokens, 44);
        assert_eq!(b.reasoning_tokens, 5); // None turn contributes 0, not skipped/dropped
    }

    #[test]
    fn total_sums_every_category() {
        let b = TokenBreakdown {
            input_tokens: 1,
            output_tokens: 2,
            cache_read_tokens: 3,
            cache_write_tokens: 4,
            reasoning_tokens: 5,
        };
        assert_eq!(b.total(), 15);
    }

    #[test]
    fn primary_model_is_first_turn_with_a_model_set() {
        let session = SessionRecord {
            session_id: "s".into(),
            project_path: "/p".into(),
            started_at: Utc::now(),
            ended_at: Utc::now(),
            turns: vec![turn(0, 0, 0, 0, None, None), turn(0, 0, 0, 0, None, Some("claude-opus-4-8"))],
        };
        assert_eq!(session.primary_model(), Some("claude-opus-4-8"));
    }

    #[test]
    fn empty_session_breakdown_is_zero() {
        let session = SessionRecord {
            session_id: "s".into(),
            project_path: "/p".into(),
            started_at: Utc::now(),
            ended_at: Utc::now(),
            turns: vec![],
        };
        assert_eq!(session.breakdown().total(), 0);
        assert_eq!(session.primary_model(), None);
    }
}
