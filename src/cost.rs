use crate::model::TokenBreakdown;
use crate::pricing::{find_model, ModelPricing};

#[derive(Debug, Clone)]
pub struct CostEstimate {
    pub model_id: String,
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_write_cost: f64,
    pub cache_read_cost: f64,
}

impl CostEstimate {
    pub fn total(&self) -> f64 {
        self.input_cost + self.output_cost + self.cache_write_cost + self.cache_read_cost
    }
}

pub fn estimate_cost(breakdown: &TokenBreakdown, pricing: &ModelPricing) -> CostEstimate {
    let per_tok = |tokens: u64, rate_per_mtok: f64| (tokens as f64 / 1_000_000.0) * rate_per_mtok;
    CostEstimate {
        model_id: pricing.id.to_string(),
        input_cost: per_tok(breakdown.input_tokens, pricing.input_per_mtok),
        output_cost: per_tok(breakdown.output_tokens, pricing.output_per_mtok),
        cache_write_cost: per_tok(breakdown.cache_write_tokens, pricing.cache_write_per_mtok),
        cache_read_cost: per_tok(breakdown.cache_read_tokens, pricing.cache_read_per_mtok),
    }
}

/// Resolve a model id/alias to pricing, falling back to `None` if unknown
/// so callers can warn instead of silently using wrong numbers.
pub fn estimate_cost_for(breakdown: &TokenBreakdown, model_id: &str) -> Option<CostEstimate> {
    find_model(model_id).map(|p| estimate_cost(breakdown, p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_cost_matches_rate_table() {
        let breakdown = TokenBreakdown {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            cache_read_tokens: 1_000_000,
            cache_write_tokens: 1_000_000,
            reasoning_tokens: 0,
        };
        let estimate = estimate_cost_for(&breakdown, "claude-sonnet-5").unwrap();
        assert_eq!(estimate.input_cost, 3.0);
        assert_eq!(estimate.output_cost, 15.0);
        assert_eq!(estimate.cache_write_cost, 3.75);
        assert_eq!(estimate.cache_read_cost, 0.30);
        assert!((estimate.total() - 22.05).abs() < 1e-9);
    }

    #[test]
    fn estimate_cost_zero_tokens_is_zero() {
        let breakdown = TokenBreakdown::default();
        let estimate = estimate_cost_for(&breakdown, "claude-opus-4-8").unwrap();
        assert_eq!(estimate.total(), 0.0);
    }

    #[test]
    fn unknown_model_returns_none() {
        let breakdown = TokenBreakdown::default();
        assert!(estimate_cost_for(&breakdown, "not-a-real-model").is_none());
    }

    #[test]
    fn model_alias_resolves_via_contains() {
        let breakdown = TokenBreakdown {
            input_tokens: 1_000_000,
            ..Default::default()
        };
        // e.g. "claude-sonnet-5-20260101" should still resolve to "claude-sonnet-5" pricing
        let estimate = estimate_cost_for(&breakdown, "claude-sonnet-5-20260101").unwrap();
        assert_eq!(estimate.input_cost, 3.0);
    }
}
