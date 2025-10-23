use anyhow::Result;
use log::{info, error};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::integration::IntegrationEngine;

/// Manages scheduled synchronization
pub struct SyncManager {
    engine: Arc<RwLock<IntegrationEngine>>,
    scheduler: JobScheduler,
}

impl SyncManager {
    /// Create a new sync manager
    pub async fn new(engine: IntegrationEngine, cron_expression: String) -> Result<Self> {
        info!("Initializing sync manager with schedule: {}", cron_expression);

        let scheduler = JobScheduler::new().await?;
        let engine = Arc::new(RwLock::new(engine));

        let mut manager = Self {
            engine,
            scheduler,
        };

        manager.setup_scheduled_job(cron_expression).await?;

        Ok(manager)
    }

    /// Setup the scheduled job
    async fn setup_scheduled_job(&mut self, cron_expression: String) -> Result<()> {
        let engine = self.engine.clone();

        let job = Job::new_async(cron_expression.as_str(), move |_uuid, _l| {
            let engine = engine.clone();
            Box::pin(async move {
                info!("🔄 Scheduled integration run starting...");

                let engine = engine.read().await;
                match engine.run_integration().await {
                    Ok(result) => {
                        info!("✅ Scheduled integration completed successfully");
                        info!("   Fetched: {}, Qualified: {}, Synced: {}",
                              result.properties_fetched,
                              result.leads_qualified,
                              result.leads_synced);
                    }
                    Err(e) => {
                        error!("❌ Scheduled integration failed: {}", e);
                    }
                }
            })
        })?;

        self.scheduler.add(job).await?;
        info!("Scheduled job configured successfully");

        Ok(())
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        info!("Starting sync scheduler");
        self.scheduler.start().await?;
        info!("Sync scheduler started successfully");
        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping sync scheduler");
        self.scheduler.shutdown().await?;
        info!("Sync scheduler stopped");
        Ok(())
    }

    /// Run integration manually (on-demand)
    pub async fn run_manual(&self) -> Result<()> {
        info!("🔄 Manual integration run triggered");

        let engine = self.engine.read().await;
        match engine.run_integration().await {
            Ok(result) => {
                info!("✅ Manual integration completed successfully");
                info!("   Fetched: {}, Qualified: {}, Synced: {}",
                      result.properties_fetched,
                      result.leads_qualified,
                      result.leads_synced);
                Ok(())
            }
            Err(e) => {
                error!("❌ Manual integration failed: {}", e);
                Err(e)
            }
        }
    }
}
