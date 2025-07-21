use chrono::{Utc, Duration};
use std::collections::HashMap;

use super::models::*;
use super::calculator::CostCalculator;

/// Aggregates usage data into various analytics views
pub struct UsageAggregator {
    cost_calculator: CostCalculator,
}

impl UsageAggregator {
    pub fn new() -> Self {
        Self {
            cost_calculator: CostCalculator::new(),
        }
    }

    /// Filter entries by time range
    pub fn filter_by_time_range(&self, entries: &[UsageEntry], time_range: TimeRange) -> Vec<UsageEntry> {
        let now = Utc::now();
        println!("🕐 Current time: {}", now.format("%Y-%m-%d %H:%M:%S"));
        
        match time_range {
            TimeRange::AllTime => {
                println!("📊 TimeRange::AllTime - returning all {} entries", entries.len());
                entries.to_vec()
            }
            TimeRange::Last7Days => {
                let cutoff = now - Duration::days(7);
                println!("📊 TimeRange::Last7Days - filtering entries after {}", cutoff.format("%Y-%m-%d"));
                let filtered = entries.iter()
                    .filter(|e| e.timestamp >= cutoff)
                    .cloned()
                    .collect::<Vec<_>>();
                println!("📊 Filtered from {} to {} entries", entries.len(), filtered.len());
                filtered
            }
            TimeRange::Last30Days => {
                let cutoff = now - Duration::days(30);
                println!("📊 TimeRange::Last30Days - filtering entries after {}", cutoff.format("%Y-%m-%d"));
                let filtered = entries.iter()
                    .filter(|e| e.timestamp >= cutoff)
                    .cloned()
                    .collect::<Vec<_>>();
                println!("📊 Filtered from {} to {} entries", entries.len(), filtered.len());
                filtered
            }
        }
    }

    /// Calculate overall usage statistics with all breakdowns pre-computed
    pub fn calculate_usage_stats(&self, entries: &[UsageEntry]) -> UsageStats {
        if entries.is_empty() {
            return UsageStats::new();
        }

        println!("🔄 Computing analytics for {} entries...", entries.len());
        
        let total_cost = entries.iter().map(|e| e.cost).sum();
        let total_input_tokens = entries.iter().map(|e| e.input_tokens as u64).sum();
        let total_output_tokens = entries.iter().map(|e| e.output_tokens as u64).sum();
        let total_cache_read_tokens = entries.iter().map(|e| e.cache_read_tokens as u64).sum();
        let total_cache_creation_tokens = entries.iter().map(|e| e.cache_creation_tokens as u64).sum();
        
        let total_tokens = total_input_tokens + total_output_tokens + total_cache_read_tokens + total_cache_creation_tokens;
        
        // Count unique sessions
        let session_count = entries.iter()
            .filter_map(|e| e.session_id.as_ref())
            .collect::<std::collections::HashSet<_>>()
            .len();

        println!("📊 Computing model stats...");
        let model_stats_vec = self.calculate_model_stats(entries);
        
        println!("📂 Computing project stats...");
        let project_stats_vec = self.calculate_project_stats(entries);
        
        println!("🔗 Computing session stats...");
        let session_stats_vec = self.calculate_session_stats(entries);
        
        println!("📅 Computing daily usage...");
        let daily_usage_vec = self.calculate_daily_usage(entries);

        // Convert to hashmaps for faster lookups
        let mut model_stats = HashMap::new();
        for stat in model_stats_vec {
            model_stats.insert(stat.model.clone(), stat);
        }

        let mut project_stats = HashMap::new();
        for stat in project_stats_vec {
            project_stats.insert(stat.project_path.clone(), stat);
        }

        let mut session_stats = HashMap::new();
        for stat in session_stats_vec {
            session_stats.insert(stat.session_id.clone(), stat);
        }

        let mut daily_usage = HashMap::new();
        for stat in daily_usage_vec {
            daily_usage.insert(stat.date.clone(), stat);
        }

        println!("✅ Analytics computation complete");

        UsageStats {
            total_cost,
            total_input_tokens,
            total_output_tokens,
            total_cache_read_tokens,
            total_cache_creation_tokens,
            total_tokens,
            session_count,
            entries: entries.to_vec(),
            model_stats,
            project_stats,
            session_stats,
            daily_usage,
        }
    }

    // Removed unused calculate_quick_stats method during cleanup

    /// Calculate model-wise statistics
    pub fn calculate_model_stats(&self, entries: &[UsageEntry]) -> Vec<ModelStats> {
        let mut model_map: HashMap<String, ModelStats> = HashMap::new();

        for entry in entries {
            let model_stat = model_map.entry(entry.model.clone()).or_insert_with(|| {
                ModelStats {
                    model: entry.model.clone(),
                    display_name: self.cost_calculator.get_model_display_name(&entry.model),
                    total_cost: 0.0,
                    total_tokens: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    request_count: 0,
                }
            });

            model_stat.total_cost += entry.cost;
            model_stat.input_tokens += entry.input_tokens as u64;
            model_stat.output_tokens += entry.output_tokens as u64;
            model_stat.cache_read_tokens += entry.cache_read_tokens as u64;
            model_stat.cache_creation_tokens += entry.cache_creation_tokens as u64;
            model_stat.total_tokens = model_stat.input_tokens + model_stat.output_tokens;
            model_stat.request_count += 1;
        }

        let mut model_stats: Vec<ModelStats> = model_map.into_values().collect();
        model_stats.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap_or(std::cmp::Ordering::Equal));
        model_stats
    }

    /// Calculate project-wise statistics
    pub fn calculate_project_stats(&self, entries: &[UsageEntry]) -> Vec<ProjectStats> {
        let mut project_map: HashMap<String, ProjectStats> = HashMap::new();

        for entry in entries {
            let project_path = entry.project_path.clone().unwrap_or_else(|| "Unknown Project".to_string());
            let project_name = self.extract_project_name(&project_path);

            let project_stat = project_map.entry(project_path.clone()).or_insert_with(|| {
                ProjectStats {
                    project_name: project_name.clone(),
                    project_path: project_path.clone(),
                    total_cost: 0.0,
                    total_tokens: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    request_count: 0,
                    session_count: 0,
                    last_used: entry.timestamp,
                }
            });

            project_stat.total_cost += entry.cost;
            project_stat.input_tokens += entry.input_tokens as u64;
            project_stat.output_tokens += entry.output_tokens as u64;
            project_stat.cache_read_tokens += entry.cache_read_tokens as u64;
            project_stat.cache_creation_tokens += entry.cache_creation_tokens as u64;
            project_stat.total_tokens = project_stat.input_tokens + project_stat.output_tokens + project_stat.cache_read_tokens + project_stat.cache_creation_tokens;
            project_stat.request_count += 1;

            if entry.timestamp > project_stat.last_used {
                project_stat.last_used = entry.timestamp;
            }
        }

        // Count unique sessions per project
        for project_stat in project_map.values_mut() {
            let sessions: std::collections::HashSet<String> = entries.iter()
                .filter(|e| e.project_path.as_ref() == Some(&project_stat.project_path))
                .filter_map(|e| e.session_id.as_ref())
                .cloned()
                .collect();
            project_stat.session_count = sessions.len();
        }

        let mut project_stats: Vec<ProjectStats> = project_map.into_values().collect();
        project_stats.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap_or(std::cmp::Ordering::Equal));
        project_stats
    }

    /// Calculate session-wise statistics
    pub fn calculate_session_stats(&self, entries: &[UsageEntry]) -> Vec<SessionStats> {
        let mut session_map: HashMap<String, SessionStats> = HashMap::new();

        for entry in entries {
            let session_key = format!(
                "{}:{}",
                entry.project_path.as_deref().unwrap_or("unknown"),
                entry.session_id.as_deref().unwrap_or("unknown")
            );

            let session_stat = session_map.entry(session_key).or_insert_with(|| {
                SessionStats {
                    session_id: entry.session_id.clone().unwrap_or_else(|| "Unknown".to_string()),
                    project_path: entry.project_path.clone().unwrap_or_else(|| "Unknown Project".to_string()),
                    total_cost: 0.0,
                    total_tokens: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    request_count: 0,
                    timestamp: entry.timestamp,
                }
            });

            session_stat.total_cost += entry.cost;
            session_stat.input_tokens += entry.input_tokens as u64;
            session_stat.output_tokens += entry.output_tokens as u64;
            session_stat.cache_read_tokens += entry.cache_read_tokens as u64;
            session_stat.cache_creation_tokens += entry.cache_creation_tokens as u64;
            session_stat.total_tokens = session_stat.input_tokens + session_stat.output_tokens + session_stat.cache_read_tokens + session_stat.cache_creation_tokens;
            session_stat.request_count += 1;

            if entry.timestamp > session_stat.timestamp {
                session_stat.timestamp = entry.timestamp;
            }
        }

        let mut session_stats: Vec<SessionStats> = session_map.into_values().collect();
        session_stats.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        session_stats
    }

    /// Calculate daily usage for timeline
    pub fn calculate_daily_usage(&self, entries: &[UsageEntry]) -> Vec<DailyUsage> {
        let mut daily_map: HashMap<String, DailyUsage> = HashMap::new();

        for entry in entries {
            let date_key = entry.timestamp.format("%Y-%m-%d").to_string();

            let daily_stat = daily_map.entry(date_key.clone()).or_insert_with(|| {
                DailyUsage {
                    date: date_key.clone(),
                    total_cost: 0.0,
                    total_tokens: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_creation_tokens: 0,
                    request_count: 0,
                    models_used: Vec::new(),
                }
            });

            daily_stat.total_cost += entry.cost;
            daily_stat.input_tokens += entry.input_tokens as u64;
            daily_stat.output_tokens += entry.output_tokens as u64;
            daily_stat.cache_read_tokens += entry.cache_read_tokens as u64;
            daily_stat.cache_creation_tokens += entry.cache_creation_tokens as u64;
            daily_stat.total_tokens = daily_stat.input_tokens + daily_stat.output_tokens + daily_stat.cache_read_tokens + daily_stat.cache_creation_tokens;
            daily_stat.request_count += 1;

            if !daily_stat.models_used.contains(&entry.model) {
                daily_stat.models_used.push(entry.model.clone());
            }
        }

        let mut daily_stats: Vec<DailyUsage> = daily_map.into_values().collect();
        daily_stats.sort_by(|a, b| a.date.cmp(&b.date));
        daily_stats
    }

    /// Calculate average cost per session
    #[allow(dead_code)] // Feature planned for future implementation
    pub fn calculate_avg_cost_per_session(&self, stats: &UsageStats) -> f64 {
        if stats.session_count == 0 {
            0.0
        } else {
            stats.total_cost / stats.session_count as f64
        }
    }

    /// Count active days
    #[allow(dead_code)] // Feature planned for future implementation
    pub fn count_active_days(&self, entries: &[UsageEntry]) -> usize {
        let unique_dates: std::collections::HashSet<String> = entries.iter()
            .map(|e| e.timestamp.format("%Y-%m-%d").to_string())
            .collect();
        unique_dates.len()
    }

    /// Calculate average daily cost
    #[allow(dead_code)] // Feature planned for future implementation
    pub fn calculate_avg_daily_cost(&self, entries: &[UsageEntry]) -> f64 {
        let active_days = self.count_active_days(entries);
        if active_days == 0 {
            0.0
        } else {
            let total_cost: f64 = entries.iter().map(|e| e.cost).sum();
            total_cost / active_days as f64
        }
    }

    /// Extract project name from path
    fn extract_project_name(&self, project_path: &str) -> String {
        // Split the path into components
        let components: Vec<&str> = project_path.split('/').filter(|s| !s.is_empty()).collect();
        
        // Try to find common project directory patterns
        let project_markers = ["Github", "github", "Projects", "projects", "code", "Code", "dev", "Development", "src", "repos"];
        
        for (i, component) in components.iter().enumerate() {
            if project_markers.contains(component) && i + 1 < components.len() {
                // Return the directory after the marker
                return components[i + 1].to_string();
            }
        }
        
        // Fallback: If no marker found, use the last non-trivial component
        // But avoid common subdirectories like "src", "scripts", "lib", etc.
        let common_subdirs = ["src", "scripts", "lib", "bin", "dist", "build", "out", "target"];
        
        // Work backwards to find a good project name
        for component in components.iter().rev() {
            if !common_subdirs.contains(component) {
                return component.to_string();
            }
        }
        
        // Ultimate fallback
        components.last().unwrap_or(&"Unknown").to_string()
    }
    
    /// Alias for calculate_usage_stats (compatibility)
    pub fn aggregate_entries(&self, entries: Vec<UsageEntry>) -> UsageStats {
        self.calculate_usage_stats(&entries)
    }
}