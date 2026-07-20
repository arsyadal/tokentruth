use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::model::{Role, SessionRecord, TurnUsage};

/// Claude Code encodes a project's absolute path into its transcript
/// directory name by replacing every path separator with `-`.
pub fn encode_project_path(path: &Path) -> String {
    path.to_string_lossy().replace('/', "-")
}

pub fn claude_projects_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".claude").join("projects"))
}

pub fn project_transcript_dir(project_path: &Path) -> Result<PathBuf> {
    Ok(claude_projects_dir()?.join(encode_project_path(project_path)))
}

/// Find a transcript file: by explicit session UUID, or the most recently
/// modified `.jsonl` in the current project's transcript directory.
pub fn find_transcript(project_path: &Path, session_id: Option<&str>) -> Result<PathBuf> {
    let dir = project_transcript_dir(project_path)?;
    if !dir.exists() {
        bail!(
            "no Claude Code transcripts found for this project at {}\n(looked in {})",
            project_path.display(),
            dir.display()
        );
    }

    if let Some(id) = session_id {
        let candidate = dir.join(format!("{id}.jsonl"));
        if !candidate.exists() {
            bail!("session {id} not found in {}", dir.display());
        }
        return Ok(candidate);
    }

    let mut best: Option<(PathBuf, std::time::SystemTime)> = None;
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        let modified = entry.metadata()?.modified()?;
        if best.as_ref().map(|(_, t)| modified > *t).unwrap_or(true) {
            best = Some((path, modified));
        }
    }

    best.map(|(p, _)| p)
        .with_context(|| format!("no .jsonl transcripts found in {}", dir.display()))
}

/// Stream-parse a transcript JSONL file into a SessionRecord.
/// Unknown fields and malformed lines are skipped rather than causing a
/// hard failure, since the transcript format is internal to Claude Code
/// and can change between releases without notice.
pub fn parse_transcript(path: &Path, project_path: &Path) -> Result<SessionRecord> {
    let file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let reader = BufReader::new(file);

    let session_id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut turns = Vec::new();
    let mut started_at: Option<DateTime<Utc>> = None;
    let mut ended_at: Option<DateTime<Utc>> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            _ => continue,
        };

        let value: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue, // skip malformed line, don't abort the whole session
        };

        let Some(turn) = turn_from_value(&value) else {
            continue;
        };

        started_at = Some(started_at.map_or(turn.timestamp, |s| s.min(turn.timestamp)));
        ended_at = Some(ended_at.map_or(turn.timestamp, |e| e.max(turn.timestamp)));
        turns.push(turn);
    }

    let now = Utc::now();
    Ok(SessionRecord {
        session_id,
        project_path: project_path.to_string_lossy().to_string(),
        started_at: started_at.unwrap_or(now),
        ended_at: ended_at.unwrap_or(now),
        turns,
    })
}

fn turn_from_value(value: &Value) -> Option<TurnUsage> {
    let entry_type = value.get("type")?.as_str()?;

    let role = match entry_type {
        "assistant" => Role::Assistant,
        "system" => Role::System,
        "user" => {
            if is_tool_result(value) {
                Role::ToolResult
            } else {
                Role::User
            }
        }
        _ => return None, // e.g. "summary" entries carry no usage
    };

    let uuid = value
        .get("uuid")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let timestamp = value
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let message = value.get("message");
    let usage = message.and_then(|m| m.get("usage"));

    let input_tokens = usage
        .and_then(|u| u.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output_tokens = usage
        .and_then(|u| u.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cache_write_tokens = usage
        .and_then(|u| u.get("cache_creation_input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cache_read_tokens = usage
        .and_then(|u| u.get("cache_read_input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let reasoning_tokens = usage
        .and_then(|u| u.get("thinking_tokens").or_else(|| u.get("reasoning_tokens")))
        .and_then(|v| v.as_u64());

    let model = message
        .and_then(|m| m.get("model"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let tool_name = message
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
        .and_then(|blocks| {
            blocks.iter().find_map(|b| {
                if b.get("type").and_then(|t| t.as_str()) == Some("tool_use") {
                    b.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            })
        });

    Some(TurnUsage {
        turn_id: uuid,
        role,
        timestamp,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
        reasoning_tokens,
        model,
        tool_name,
    })
}

fn is_tool_result(value: &Value) -> bool {
    value
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_array())
        .map(|blocks| {
            blocks
                .iter()
                .any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result"))
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn encode_project_path_replaces_slashes() {
        let path = Path::new("/Users/macintoshhd/Documents/OBE/OBE-FRI-BE");
        assert_eq!(
            encode_project_path(path),
            "-Users-macintoshhd-Documents-OBE-OBE-FRI-BE"
        );
    }

    #[test]
    fn turn_from_value_parses_assistant_usage() {
        let value = json!({
            "type": "assistant",
            "uuid": "abc-123",
            "timestamp": "2026-07-17T04:25:36.680Z",
            "message": {
                "role": "assistant",
                "model": "claude-sonnet-5",
                "usage": {
                    "input_tokens": 2,
                    "output_tokens": 290,
                    "cache_creation_input_tokens": 28759,
                    "cache_read_input_tokens": 100
                },
                "content": [
                    {"type": "tool_use", "name": "Read", "input": {}}
                ]
            }
        });
        let turn = turn_from_value(&value).expect("should parse");
        assert_eq!(turn.role, Role::Assistant);
        assert_eq!(turn.input_tokens, 2);
        assert_eq!(turn.output_tokens, 290);
        assert_eq!(turn.cache_write_tokens, 28759);
        assert_eq!(turn.cache_read_tokens, 100);
        assert_eq!(turn.model.as_deref(), Some("claude-sonnet-5"));
        assert_eq!(turn.tool_name.as_deref(), Some("Read"));
    }

    #[test]
    fn turn_from_value_detects_tool_result_as_role() {
        let value = json!({
            "type": "user",
            "uuid": "def-456",
            "timestamp": "2026-07-17T04:25:37.983Z",
            "message": {
                "role": "user",
                "content": [
                    {"type": "tool_result", "tool_use_id": "x", "content": "ok"}
                ]
            }
        });
        let turn = turn_from_value(&value).expect("should parse");
        assert_eq!(turn.role, Role::ToolResult);
    }

    #[test]
    fn turn_from_value_skips_summary_entries() {
        let value = json!({"type": "summary", "summary": "compacted history"});
        assert!(turn_from_value(&value).is_none());
    }

    #[test]
    fn turn_from_value_skips_malformed_entries() {
        let value = json!({"no_type_field": true});
        assert!(turn_from_value(&value).is_none());
    }
}
