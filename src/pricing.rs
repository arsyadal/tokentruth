/// Static, bundled pricing table (USD per million tokens).
/// Updated manually per release — no network calls in the MVP.
/// Source: published provider pricing pages as of 2026-07-20.
pub struct ModelPricing {
    pub id: &'static str,
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub cache_write_per_mtok: f64,
    pub cache_read_per_mtok: f64,
}

pub const PRICING_TABLE: &[ModelPricing] = &[
    ModelPricing {
        id: "claude-opus-4-8",
        input_per_mtok: 15.0,
        output_per_mtok: 75.0,
        cache_write_per_mtok: 18.75,
        cache_read_per_mtok: 1.50,
    },
    ModelPricing {
        id: "claude-sonnet-5",
        input_per_mtok: 3.0,
        output_per_mtok: 15.0,
        cache_write_per_mtok: 3.75,
        cache_read_per_mtok: 0.30,
    },
    ModelPricing {
        id: "claude-haiku-4-5",
        input_per_mtok: 0.80,
        output_per_mtok: 4.0,
        cache_write_per_mtok: 1.0,
        cache_read_per_mtok: 0.08,
    },
    ModelPricing {
        id: "gpt-5",
        input_per_mtok: 5.0,
        output_per_mtok: 15.0,
        cache_write_per_mtok: 5.0,
        cache_read_per_mtok: 0.50,
    },
    ModelPricing {
        id: "gemini-3-pro",
        input_per_mtok: 2.50,
        output_per_mtok: 10.0,
        cache_write_per_mtok: 2.50,
        cache_read_per_mtok: 0.25,
    },
];

pub fn find_model(id: &str) -> Option<&'static ModelPricing> {
    let needle = id.to_lowercase();
    PRICING_TABLE
        .iter()
        .find(|m| m.id.to_lowercase() == needle || needle.contains(&m.id.to_lowercase()))
}
