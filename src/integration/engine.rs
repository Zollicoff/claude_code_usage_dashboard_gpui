use anyhow::{Context, Result};
use log::{info, warn, error};
use uuid::Uuid;

use crate::{
    api::{PropStreamClient, PipedriveClient},
    config::AppConfig,
    db::LeadRepository,
    filters::{LeadQualifier, LeadScorer},
    models::{QualifiedLead, SyncStatus, IntegrationStats},
};

/// Main integration engine orchestrating the entire process
pub struct IntegrationEngine {
    propstream: PropStreamClient,
    pipedrive: PipedriveClient,
    qualifier: LeadQualifier,
    db: LeadRepository,
    config: AppConfig,
}

impl IntegrationEngine {
    /// Create a new integration engine
    pub async fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing integration engine");

        config.validate()?;

        let propstream = PropStreamClient::new(config.propstream.api_key.clone());
        let pipedrive = PipedriveClient::new(config.pipedrive.api_token.clone());
        let qualifier = LeadQualifier::new(config.criteria.clone());
        let db = LeadRepository::new(&config.database.url).await?;

        Ok(Self {
            propstream,
            pipedrive,
            qualifier,
            db,
            config,
        })
    }

    /// Run the full integration process
    pub async fn run_integration(&self) -> Result<IntegrationRunResult> {
        let run_id = Uuid::new_v4().to_string();
        info!("Starting integration run: {}", run_id);

        let mut result = IntegrationRunResult::default();

        // Step 1: Fetch properties from PropStream
        info!("Step 1: Fetching properties from PropStream");
        let properties = match self.fetch_properties().await {
            Ok(props) => {
                info!("Fetched {} properties", props.len());
                result.properties_fetched = props.len() as u64;
                props
            }
            Err(e) => {
                error!("Failed to fetch properties: {}", e);
                result.status = "Failed".to_string();
                result.error_message = Some(e.to_string());
                return Ok(result);
            }
        };

        if properties.is_empty() {
            warn!("No properties fetched, ending run");
            result.status = "Completed".to_string();
            return Ok(result);
        }

        // Step 2: Qualify leads
        info!("Step 2: Qualifying leads");
        let qualified = self.qualifier.qualify_batch(properties);
        let qualified_leads: Vec<_> = qualified
            .into_iter()
            .filter_map(|(prop, qual_result)| {
                if qual_result.passed {
                    let score = LeadScorer::score(&prop);
                    Some(QualifiedLead::new(prop, score))
                } else {
                    None
                }
            })
            .collect();

        info!("Qualified {} leads", qualified_leads.len());
        result.leads_qualified = qualified_leads.len() as u64;

        // Step 3: Filter duplicates
        info!("Step 3: Checking for duplicates");
        let mut new_leads = Vec::new();
        for lead in qualified_leads {
            if self.db
                .is_duplicate(
                    &lead.property.address.full_address,
                    &lead.property.owner.name,
                )
                .await?
            {
                warn!("Duplicate lead: {}", lead.property.address.full_address);
                result.duplicates_skipped += 1;

                let mut dup_lead = lead;
                dup_lead.sync_status = SyncStatus::Duplicate;
                self.db.save_lead(&dup_lead).await?;
            } else {
                new_leads.push(lead);
            }
        }

        info!("{} new leads to sync", new_leads.len());

        // Step 4: Save leads to database
        info!("Step 4: Saving leads to database");
        for lead in &new_leads {
            self.db.save_lead(lead).await?;
        }

        // Step 5: Sync to Pipedrive
        info!("Step 5: Syncing leads to Pipedrive");
        for mut lead in new_leads {
            match self.sync_lead_to_pipedrive(&lead).await {
                Ok(pipedrive_id) => {
                    info!("Successfully synced lead: {}", pipedrive_id);
                    lead.pipedrive_id = Some(pipedrive_id.clone());
                    lead.sync_status = SyncStatus::Synced;
                    self.db
                        .update_sync_status(&lead.id, SyncStatus::Synced, Some(pipedrive_id))
                        .await?;
                    result.leads_synced += 1;
                }
                Err(e) => {
                    error!("Failed to sync lead {}: {}", lead.id, e);
                    let error_msg = e.to_string();
                    self.db
                        .update_sync_status(
                            &lead.id,
                            SyncStatus::Failed(error_msg),
                            None,
                        )
                        .await?;
                    result.errors_count += 1;
                }
            }
        }

        // Step 6: Update statistics
        info!("Step 6: Updating statistics");
        self.update_statistics(&result).await?;

        // Record sync run
        self.db
            .record_sync_run(
                &run_id,
                result.properties_fetched,
                result.leads_qualified,
                result.leads_synced,
                result.duplicates_skipped,
                result.errors_count,
                if result.errors_count == 0 { "Success" } else { "Partial Success" },
                result.error_message.clone(),
            )
            .await?;

        result.status = "Completed".to_string();
        info!("Integration run completed: {}", run_id);
        info!("Results - Fetched: {}, Qualified: {}, Synced: {}, Duplicates: {}, Errors: {}",
              result.properties_fetched,
              result.leads_qualified,
              result.leads_synced,
              result.duplicates_skipped,
              result.errors_count);

        Ok(result)
    }

    /// Fetch properties from PropStream
    async fn fetch_properties(&self) -> Result<Vec<crate::models::Property>> {
        let criteria = &self.config.criteria;

        let property_types: Vec<String> = criteria
            .property_types
            .included_types
            .iter()
            .map(|pt| format!("{:?}", pt))
            .collect();

        self.propstream
            .fetch_properties(
                criteria.geographic.states.clone(),
                criteria.geographic.counties.clone(),
                property_types,
                Some(criteria.financial.min_equity_percent),
                Some(criteria.financial.max_equity_percent),
                Some(criteria.owner.require_absentee),
                Some(criteria.owner.min_ownership_years),
                self.config.propstream.max_results_per_run,
            )
            .await
    }

    /// Sync a single lead to Pipedrive
    async fn sync_lead_to_pipedrive(&self, lead: &QualifiedLead) -> Result<String> {
        // Check if already exists in Pipedrive
        if self.pipedrive
            .check_duplicate(&lead.property.address.full_address)
            .await?
        {
            return Err(anyhow::anyhow!("Lead already exists in Pipedrive"));
        }

        self.pipedrive.create_lead(lead).await
    }

    /// Update integration statistics
    async fn update_statistics(&self, result: &IntegrationRunResult) -> Result<()> {
        let mut stats = self.db.get_stats().await?;

        stats.total_properties_fetched += result.properties_fetched;
        stats.total_qualified_leads += result.leads_qualified;
        stats.total_synced_to_pipedrive += result.leads_synced;
        stats.total_duplicates_skipped += result.duplicates_skipped;
        stats.total_failures += result.errors_count;
        stats.last_run = Some(chrono::Utc::now());

        if result.errors_count == 0 {
            stats.last_success = Some(chrono::Utc::now());
            stats.last_error = None;
        } else {
            stats.last_error = result.error_message.clone();
        }

        stats.update_qualification_rate();
        stats.update_sync_success_rate();

        self.db.update_stats(&stats).await?;

        Ok(())
    }

    /// Test all API connections
    pub async fn test_connections(&self) -> Result<ConnectionTestResult> {
        info!("Testing API connections");

        let mut result = ConnectionTestResult::default();

        // Test PropStream
        match self.propstream.test_connection().await {
            Ok(true) => {
                info!("PropStream connection: OK");
                result.propstream_ok = true;
            }
            Ok(false) | Err(_) => {
                error!("PropStream connection: FAILED");
                result.propstream_ok = false;
            }
        }

        // Test Pipedrive
        match self.pipedrive.test_connection().await {
            Ok(true) => {
                info!("Pipedrive connection: OK");
                result.pipedrive_ok = true;
            }
            Ok(false) | Err(_) => {
                error!("Pipedrive connection: FAILED");
                result.pipedrive_ok = false;
            }
        }

        // Test Database
        match self.db.get_stats().await {
            Ok(_) => {
                info!("Database connection: OK");
                result.database_ok = true;
            }
            Err(_) => {
                error!("Database connection: FAILED");
                result.database_ok = false;
            }
        }

        Ok(result)
    }

    /// Get current statistics
    pub async fn get_statistics(&self) -> Result<IntegrationStats> {
        self.db.get_stats().await
    }
}

#[derive(Debug, Default)]
pub struct IntegrationRunResult {
    pub properties_fetched: u64,
    pub leads_qualified: u64,
    pub leads_synced: u64,
    pub duplicates_skipped: u64,
    pub errors_count: u64,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Default)]
pub struct ConnectionTestResult {
    pub propstream_ok: bool,
    pub pipedrive_ok: bool,
    pub database_ok: bool,
}

impl ConnectionTestResult {
    pub fn all_ok(&self) -> bool {
        self.propstream_ok && self.pipedrive_ok && self.database_ok
    }
}
