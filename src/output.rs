use comfy_table::{presets::UTF8_FULL, Cell, Table};

use crate::cost::CostEstimate;
use crate::model::{SessionRecord, TokenBreakdown};

pub fn print_breakdown_table(session: &SessionRecord, breakdown: &TokenBreakdown) {
    println!(
        "Session {}  ({} -> {})",
        session.session_id, session.started_at, session.ended_at
    );
    if let Some(model) = session.primary_model() {
        println!("Model: {model}");
    }
    println!("Turns: {}\n", session.turns.len());

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Category", "Tokens"]);
    table.add_row(vec![
        Cell::new("Input"),
        Cell::new(breakdown.input_tokens.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Output"),
        Cell::new(breakdown.output_tokens.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Cache write"),
        Cell::new(breakdown.cache_write_tokens.to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Cache read"),
        Cell::new(breakdown.cache_read_tokens.to_string()),
    ]);
    if breakdown.reasoning_tokens > 0 {
        table.add_row(vec![
            Cell::new("Reasoning"),
            Cell::new(breakdown.reasoning_tokens.to_string()),
        ]);
    }
    table.add_row(vec![
        Cell::new("Total"),
        Cell::new(breakdown.total().to_string()),
    ]);

    println!("{table}");
}

pub fn print_compare_table(
    before_label: &str,
    before: &TokenBreakdown,
    after_label: &str,
    after: &TokenBreakdown,
) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Category", before_label, after_label, "Delta", "Delta %"]);

    let rows: [(&str, u64, u64); 5] = [
        ("Input", before.input_tokens, after.input_tokens),
        ("Output", before.output_tokens, after.output_tokens),
        ("Cache write", before.cache_write_tokens, after.cache_write_tokens),
        ("Cache read", before.cache_read_tokens, after.cache_read_tokens),
        ("Total", before.total(), after.total()),
    ];

    for (label, b, a) in rows {
        let delta = a as i64 - b as i64;
        let pct = if b == 0 {
            "n/a".to_string()
        } else {
            format!("{:+.1}%", (delta as f64 / b as f64) * 100.0)
        };
        table.add_row(vec![
            Cell::new(label),
            Cell::new(b.to_string()),
            Cell::new(a.to_string()),
            Cell::new(format!("{delta:+}")),
            Cell::new(pct),
        ]);
    }

    println!("{table}");
}

pub fn print_cost_table(estimates: &[CostEstimate]) {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Model", "Input $", "Output $", "Cache W $", "Cache R $", "Total $"]);

    for e in estimates {
        table.add_row(vec![
            Cell::new(&e.model_id),
            Cell::new(format!("{:.4}", e.input_cost)),
            Cell::new(format!("{:.4}", e.output_cost)),
            Cell::new(format!("{:.4}", e.cache_write_cost)),
            Cell::new(format!("{:.4}", e.cache_read_cost)),
            Cell::new(format!("{:.4}", e.total())),
        ]);
    }

    println!("{table}");
}
