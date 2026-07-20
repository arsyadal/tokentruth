mod cost;
mod model;
mod output;
mod parser;
mod pricing;

use std::env;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::Value;

use cost::estimate_cost_for;
use model::SessionRecord;
use output::{print_breakdown_table, print_compare_table, print_cost_table};
use parser::{find_transcript, parse_transcript};

#[derive(Parser)]
#[command(
    name = "tokentruth",
    version,
    about = "Independent verification of AI coding agent token-savings claims, computed from raw session transcripts."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Analyze one session's token usage
    Analyze {
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        breakdown: bool,
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Compare two sessions (e.g. before/after enabling a compression tool)
    Compare {
        #[arg(long)]
        before: String,
        #[arg(long)]
        after: String,
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Estimate cost across one or more models for a session
    Cost {
        #[arg(long)]
        session: Option<String>,
        #[arg(long, value_delimiter = ',')]
        models: Vec<String>,
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Export raw session data
    Export {
        #[arg(long)]
        session: Option<String>,
        #[arg(long, default_value = "json")]
        format: String,
        #[arg(long)]
        project: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Analyze {
            session,
            breakdown: _,
            project,
        } => {
            let project = resolve_project(project)?;
            warn_on_retention();
            let path = find_transcript(&project, session.as_deref())?;
            let record = parse_transcript(&path, &project)?;
            let breakdown = record.breakdown();
            print_breakdown_table(&record, &breakdown);
        }
        Command::Compare {
            before,
            after,
            project,
        } => {
            let project = resolve_project(project)?;
            warn_on_retention();
            let before_path = find_transcript(&project, Some(&before))?;
            let after_path = find_transcript(&project, Some(&after))?;
            let before_record = parse_transcript(&before_path, &project)?;
            let after_record = parse_transcript(&after_path, &project)?;
            print_compare_table(
                &before,
                &before_record.breakdown(),
                &after,
                &after_record.breakdown(),
            );
        }
        Command::Cost {
            session,
            models,
            project,
        } => {
            let project = resolve_project(project)?;
            let path = find_transcript(&project, session.as_deref())?;
            let record = parse_transcript(&path, &project)?;
            let breakdown = record.breakdown();

            let model_ids: Vec<String> = if models.is_empty() {
                record
                    .primary_model()
                    .map(|m| vec![m.to_string()])
                    .unwrap_or_else(|| vec!["claude-sonnet-5".to_string()])
            } else {
                models
            };

            let mut estimates = Vec::new();
            for id in &model_ids {
                match estimate_cost_for(&breakdown, id) {
                    Some(e) => estimates.push(e),
                    None => eprintln!("warning: unknown model '{id}', skipping"),
                }
            }
            print_cost_table(&estimates);
        }
        Command::Export {
            session,
            format,
            project,
        } => {
            let project = resolve_project(project)?;
            let path = find_transcript(&project, session.as_deref())?;
            let record = parse_transcript(&path, &project)?;
            export_session(&record, &format)?;
        }
    }

    Ok(())
}

fn resolve_project(explicit: Option<PathBuf>) -> Result<PathBuf> {
    match explicit {
        Some(p) => Ok(p),
        None => Ok(env::current_dir()?),
    }
}

fn export_session(record: &SessionRecord, format: &str) -> Result<()> {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(record)?);
        }
        "csv" => {
            println!("turn_id,role,timestamp,input_tokens,output_tokens,cache_read_tokens,cache_write_tokens,reasoning_tokens,model,tool_name");
            for t in &record.turns {
                println!(
                    "{},{:?},{},{},{},{},{},{},{},{}",
                    t.turn_id,
                    t.role,
                    t.timestamp,
                    t.input_tokens,
                    t.output_tokens,
                    t.cache_read_tokens,
                    t.cache_write_tokens,
                    t.reasoning_tokens.unwrap_or(0),
                    t.model.as_deref().unwrap_or(""),
                    t.tool_name.as_deref().unwrap_or(""),
                );
            }
        }
        other => anyhow::bail!("unknown export format '{other}', expected json or csv"),
    }
    Ok(())
}

/// Claude Code's default settings can auto-delete transcripts after ~30
/// days. Warn the user if their settings.json doesn't override this, since
/// a deleted transcript means an incomplete or impossible audit.
fn warn_on_retention() {
    let Some(home) = dirs::home_dir() else { return };
    let settings_path = home.join(".claude").join("settings.json");
    let Ok(contents) = std::fs::read_to_string(&settings_path) else {
        eprintln!(
            "note: no ~/.claude/settings.json found — Claude Code's default ~30 day transcript retention applies. \
             Set \"cleanupPeriodDays\" there if you need sessions kept longer for auditing."
        );
        return;
    };
    let Ok(json): Result<Value, _> = serde_json::from_str(&contents) else {
        return;
    };
    if json.get("cleanupPeriodDays").is_none() {
        eprintln!(
            "note: ~/.claude/settings.json has no \"cleanupPeriodDays\" override — default ~30 day \
             transcript retention applies. Old sessions may already be gone."
        );
    }
}
