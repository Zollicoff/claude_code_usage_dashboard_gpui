/// Utilities for formatting numbers, currency, and dates
use chrono::{DateTime, Utc};

/// Format currency values with appropriate precision
#[allow(dead_code)] // Utility function for future features
pub fn format_currency(amount: f64) -> String {
    if amount == 0.0 {
        "$0.00".to_string()
    } else if amount < 0.01 {
        format!("${:.4}", amount)
    } else if amount < 1.0 {
        format!("${:.3}", amount)
    } else {
        format!("${:.2}", amount)
    }
}

/// Format token counts in human-readable format
#[allow(dead_code)] // Utility function for future features
pub fn format_tokens(tokens: u64) -> String {
    if tokens == 0 {
        "0".to_string()
    } else if tokens < 1_000 {
        tokens.to_string()
    } else if tokens < 1_000_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else if tokens < 1_000_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else {
        format!("{:.1}B", tokens as f64 / 1_000_000_000.0)
    }
}

/// Format percentage values
#[allow(dead_code)] // Utility function for future features
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Format timestamps for display
#[allow(dead_code)] // Utility function for future features
pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format date only
#[allow(dead_code)] // Utility function for future features
pub fn format_date(timestamp: &DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d").to_string()
}

/// Format relative time (e.g., "2 days ago")
#[allow(dead_code)] // Utility function for future features
pub fn format_relative_time(timestamp: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*timestamp);
    
    if duration.num_days() > 0 {
        format!("{} days ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes ago", duration.num_minutes())
    } else {
        "Just now".to_string()
    }
}

/// Truncate project paths for display
#[allow(dead_code)] // Utility function for future features
pub fn truncate_project_path(path: &str, max_length: usize) -> String {
    if path.len() <= max_length {
        path.to_string()
    } else {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 2 {
            format!(".../{}", parts[parts.len() - 1])
        } else {
            format!("{}...", &path[..max_length - 3])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(0.0), "$0.00");
        assert_eq!(format_currency(0.001), "$0.0010");
        assert_eq!(format_currency(0.05), "$0.050");
        assert_eq!(format_currency(1.50), "$1.50");
        assert_eq!(format_currency(123.456), "$123.46");
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1_500), "1.5K");
        assert_eq!(format_tokens(2_500_000), "2.5M");
        assert_eq!(format_tokens(1_200_000_000), "1.2B");
    }

    #[test]
    fn test_truncate_project_path() {
        assert_eq!(truncate_project_path("short", 20), "short");
        assert_eq!(truncate_project_path("/very/long/path/to/project", 15), ".../project");
        assert_eq!(truncate_project_path("toolongname", 8), "toolo...");
    }
}