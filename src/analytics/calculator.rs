/// Cost calculator for AI models with accurate pricing
pub struct CostCalculator;

impl CostCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate cost for a usage entry
    pub fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        cache_read_tokens: u32,
        cache_creation_tokens: u32,
    ) -> f64 {
        let pricing = self.get_model_pricing(model);
        
        let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_price;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_price;
        let cache_read_cost = (cache_read_tokens as f64 / 1_000_000.0) * pricing.cache_read_price;
        let cache_write_cost = (cache_creation_tokens as f64 / 1_000_000.0) * pricing.cache_write_price;
        
        input_cost + output_cost + cache_read_cost + cache_write_cost
    }

    /// Get pricing information for a model
    fn get_model_pricing(&self, model: &str) -> ModelPricing {
        // Model pricing (per million tokens) - matching reference implementation exactly
        if model.contains("opus-4") || model.contains("claude-opus-4") {
            ModelPricing {
                input_price: 15.0,
                output_price: 75.0,
                cache_read_price: 1.50,
                cache_write_price: 18.75,
            }
        } else if model.contains("sonnet-4") || model.contains("claude-sonnet-4") {
            // Sonnet pricing from reference implementation
            ModelPricing {
                input_price: 3.0,
                output_price: 15.0,
                cache_read_price: 0.30,
                cache_write_price: 3.75,
            }
        } else {
            // Return 0 for unknown models to avoid incorrect cost estimations (like reference)
            ModelPricing {
                input_price: 0.0,
                output_price: 0.0,
                cache_read_price: 0.0,
                cache_write_price: 0.0,
            }
        }
    }

    /// Get display name for a model
    pub fn get_model_display_name(&self, model: &str) -> String {
        if model.contains("opus-4") || model.contains("claude-opus-4") {
            "Opus 4".to_string()
        } else if model.contains("sonnet-4") || model.contains("claude-sonnet-4") {
            "Sonnet 4".to_string()
        } else {
            // Return the model name as-is for unknown models
            model.to_string()
        }
    }

    /// Get color for a model (for UI display)
    #[allow(dead_code)] // Feature planned for future implementation
    pub fn get_model_color(&self, model: &str) -> &'static str {
        if model.contains("opus") {
            "#8B5CF6" // Purple for Opus
        } else if model.contains("sonnet") {
            "#3B82F6" // Blue for Sonnet
        } else if model.contains("haiku") {
            "#10B981" // Green for Haiku
        } else {
            "#6B7280" // Gray for unknown
        }
    }
}

/// Pricing information for an AI model
struct ModelPricing {
    input_price: f64,      // Per million tokens
    output_price: f64,     // Per million tokens
    cache_read_price: f64, // Per million tokens
    cache_write_price: f64, // Per million tokens
}