use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Integration statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationStats {
    pub total_properties_fetched: u64,
    pub total_qualified_leads: u64,
    pub total_synced_to_pipedrive: u64,
    pub total_duplicates_skipped: u64,
    pub total_failures: u64,
    pub last_run: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub qualification_rate: f64,
    pub sync_success_rate: f64,
}

impl IntegrationStats {
    pub fn update_qualification_rate(&mut self) {
        if self.total_properties_fetched > 0 {
            self.qualification_rate =
                (self.total_qualified_leads as f64 / self.total_properties_fetched as f64) * 100.0;
        }
    }

    pub fn update_sync_success_rate(&mut self) {
        let total_sync_attempts = self.total_synced_to_pipedrive + self.total_failures;
        if total_sync_attempts > 0 {
            self.sync_success_rate =
                (self.total_synced_to_pipedrive as f64 / total_sync_attempts as f64) * 100.0;
        }
    }
}
