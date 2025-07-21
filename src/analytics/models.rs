use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Raw usage entry from JSONL file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub project_path: Option<String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_read_tokens: u32,
    pub cache_creation_tokens: u32,
    pub cost: f64,
}

/// Aggregated statistics for the dashboard
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub total_cost: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cache_creation_tokens: u64,
    pub total_tokens: u64,
    pub session_count: usize,
    pub entries: Vec<UsageEntry>,
    pub model_stats: std::collections::HashMap<String, ModelStats>,
    pub project_stats: std::collections::HashMap<String, ProjectStats>,
    pub session_stats: std::collections::HashMap<String, SessionStats>,
    pub daily_usage: std::collections::HashMap<String, DailyUsage>,
}

impl UsageStats {
    pub fn new() -> Self {
        Self {
            total_cost: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cache_read_tokens: 0,
            total_cache_creation_tokens: 0,
            total_tokens: 0,
            session_count: 0,
            entries: Vec::new(),
            model_stats: std::collections::HashMap::new(),
            project_stats: std::collections::HashMap::new(),
            session_stats: std::collections::HashMap::new(),
            daily_usage: std::collections::HashMap::new(),
        }
    }

    #[allow(dead_code)] // Utility method for future use
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.total_cost == 0.0
    }
}

/// Model usage breakdown
#[derive(Debug, Clone)]
pub struct ModelStats {
    pub model: String,
    pub display_name: String,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub request_count: usize,
}

/// Project usage breakdown
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub project_name: String,
    pub project_path: String,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub request_count: usize,
    pub session_count: usize,
    pub last_used: DateTime<Utc>,
}

/// Session usage breakdown
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub session_id: String,
    pub project_path: String,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub request_count: usize,
    pub timestamp: DateTime<Utc>,
    // Removed unused date field during cleanup
}

/// Daily usage for timeline
#[derive(Debug, Clone)]
pub struct DailyUsage {
    pub date: String,
    pub total_cost: f64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    pub request_count: usize,
    pub models_used: Vec<String>,
}

/// Time range filter options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeRange {
    AllTime,
    Last7Days,
    Last30Days,
}

impl TimeRange {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            TimeRange::AllTime => "All Time",
            TimeRange::Last7Days => "7 Days",
            TimeRange::Last30Days => "30 Days",
        }
    }
}

