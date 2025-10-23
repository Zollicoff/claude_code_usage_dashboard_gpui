use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use log::{info, debug, error};

use crate::models::{QualifiedLead, SyncStatus, IntegrationStats};

/// Database repository for lead management
pub struct LeadRepository {
    pool: SqlitePool,
}

impl LeadRepository {
    /// Create a new repository with database connection
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .context("Failed to connect to database")?;

        let repo = Self { pool };
        repo.initialize_schema().await?;

        Ok(repo)
    }

    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema");

        let schema = include_str!("schema.sql");

        // Split and execute each statement
        for statement in schema.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed)
                    .execute(&self.pool)
                    .await
                    .context("Failed to execute schema statement")?;
            }
        }

        info!("Database schema initialized successfully");
        Ok(())
    }

    /// Check if a lead is a duplicate
    pub async fn is_duplicate(&self, address: &str, owner_name: &str) -> Result<bool> {
        debug!("Checking duplicate: {} - {}", address, owner_name);

        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM leads WHERE property_address = ? AND owner_name = ?"
        )
        .bind(address)
        .bind(owner_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(result > 0)
    }

    /// Save a qualified lead
    pub async fn save_lead(&self, lead: &QualifiedLead) -> Result<()> {
        debug!("Saving lead: {}", lead.id);

        let sync_status = match &lead.sync_status {
            SyncStatus::Pending => "Pending",
            SyncStatus::Synced => "Synced",
            SyncStatus::Failed(msg) => msg.as_str(),
            SyncStatus::Duplicate => "Duplicate",
        };

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO leads (
                id, property_id, property_address, owner_name, county, state,
                property_type, equity_percent, ownership_years, estimated_value,
                qualification_score, is_absentee, is_distressed, created_at,
                pipedrive_id, sync_status, last_synced_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&lead.id)
        .bind(&lead.property.id)
        .bind(&lead.property.address.full_address)
        .bind(&lead.property.owner.name)
        .bind(&lead.property.address.county)
        .bind(&lead.property.address.state)
        .bind(format!("{:?}", lead.property.property_details.property_type))
        .bind(lead.property.calculate_equity_percent())
        .bind(lead.property.owner.ownership_years)
        .bind(lead.property.financial.estimated_value)
        .bind(lead.qualification_score)
        .bind(lead.property.owner.is_absentee)
        .bind(lead.property.flags.is_distressed)
        .bind(lead.created_at)
        .bind(&lead.pipedrive_id)
        .bind(sync_status)
        .bind(if lead.sync_status == SyncStatus::Synced {
            Some(Utc::now())
        } else {
            None
        })
        .execute(&self.pool)
        .await
        .context("Failed to save lead")?;

        Ok(())
    }

    /// Update lead sync status
    pub async fn update_sync_status(
        &self,
        lead_id: &str,
        status: SyncStatus,
        pipedrive_id: Option<String>,
    ) -> Result<()> {
        debug!("Updating sync status for lead {}: {:?}", lead_id, status);

        let (status_str, error_msg) = match &status {
            SyncStatus::Pending => ("Pending", None),
            SyncStatus::Synced => ("Synced", None),
            SyncStatus::Failed(msg) => ("Failed", Some(msg.clone())),
            SyncStatus::Duplicate => ("Duplicate", None),
        };

        sqlx::query(
            r#"
            UPDATE leads
            SET sync_status = ?,
                sync_error = ?,
                pipedrive_id = ?,
                last_synced_at = ?
            WHERE id = ?
            "#
        )
        .bind(status_str)
        .bind(error_msg)
        .bind(pipedrive_id)
        .bind(if status == SyncStatus::Synced {
            Some(Utc::now())
        } else {
            None
        })
        .bind(lead_id)
        .execute(&self.pool)
        .await
        .context("Failed to update sync status")?;

        Ok(())
    }

    /// Get pending leads for sync
    pub async fn get_pending_leads(&self, limit: i64) -> Result<Vec<String>> {
        debug!("Fetching pending leads (limit: {})", limit);

        let ids = sqlx::query_scalar::<_, String>(
            "SELECT id FROM leads WHERE sync_status = 'Pending' LIMIT ?"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(ids)
    }

    /// Get integration statistics
    pub async fn get_stats(&self) -> Result<IntegrationStats> {
        debug!("Fetching integration statistics");

        let row = sqlx::query_as::<_, (u64, u64, u64, u64, u64, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<String>, f64, f64)>(
            r#"
            SELECT
                total_properties_fetched,
                total_qualified_leads,
                total_synced_to_pipedrive,
                total_duplicates_skipped,
                total_failures,
                last_run,
                last_success,
                last_error,
                qualification_rate,
                sync_success_rate
            FROM integration_stats WHERE id = 1
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(IntegrationStats {
            total_properties_fetched: row.0,
            total_qualified_leads: row.1,
            total_synced_to_pipedrive: row.2,
            total_duplicates_skipped: row.3,
            total_failures: row.4,
            last_run: row.5,
            last_success: row.6,
            last_error: row.7,
            qualification_rate: row.8,
            sync_success_rate: row.9,
        })
    }

    /// Update integration statistics
    pub async fn update_stats(&self, stats: &IntegrationStats) -> Result<()> {
        debug!("Updating integration statistics");

        sqlx::query(
            r#"
            UPDATE integration_stats SET
                total_properties_fetched = ?,
                total_qualified_leads = ?,
                total_synced_to_pipedrive = ?,
                total_duplicates_skipped = ?,
                total_failures = ?,
                last_run = ?,
                last_success = ?,
                last_error = ?,
                qualification_rate = ?,
                sync_success_rate = ?
            WHERE id = 1
            "#
        )
        .bind(stats.total_properties_fetched)
        .bind(stats.total_qualified_leads)
        .bind(stats.total_synced_to_pipedrive)
        .bind(stats.total_duplicates_skipped)
        .bind(stats.total_failures)
        .bind(stats.last_run)
        .bind(stats.last_success)
        .bind(&stats.last_error)
        .bind(stats.qualification_rate)
        .bind(stats.sync_success_rate)
        .execute(&self.pool)
        .await
        .context("Failed to update stats")?;

        Ok(())
    }

    /// Record a sync run in history
    pub async fn record_sync_run(
        &self,
        run_id: &str,
        properties_fetched: u64,
        leads_qualified: u64,
        leads_synced: u64,
        duplicates_skipped: u64,
        errors_count: u64,
        status: &str,
        error_message: Option<String>,
    ) -> Result<()> {
        debug!("Recording sync run: {}", run_id);

        sqlx::query(
            r#"
            INSERT INTO sync_history (
                run_id, started_at, completed_at, properties_fetched,
                leads_qualified, leads_synced, duplicates_skipped,
                errors_count, status, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(run_id)
        .bind(Utc::now())
        .bind(Utc::now())
        .bind(properties_fetched)
        .bind(leads_qualified)
        .bind(leads_synced)
        .bind(duplicates_skipped)
        .bind(errors_count)
        .bind(status)
        .bind(error_message)
        .execute(&self.pool)
        .await
        .context("Failed to record sync run")?;

        Ok(())
    }

    /// Get recent sync history
    pub async fn get_sync_history(&self, limit: i64) -> Result<Vec<SyncHistoryRecord>> {
        debug!("Fetching sync history (limit: {})", limit);

        let records = sqlx::query_as::<_, SyncHistoryRecord>(
            r#"
            SELECT
                run_id, started_at, completed_at, properties_fetched,
                leads_qualified, leads_synced, duplicates_skipped,
                errors_count, status, error_message
            FROM sync_history
            ORDER BY started_at DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct SyncHistoryRecord {
    pub run_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub properties_fetched: i64,
    pub leads_qualified: i64,
    pub leads_synced: i64,
    pub duplicates_skipped: i64,
    pub errors_count: i64,
    pub status: String,
    pub error_message: Option<String>,
}
