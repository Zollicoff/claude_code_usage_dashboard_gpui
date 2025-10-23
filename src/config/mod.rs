use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use log::info;

use crate::models::LeadCriteria;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub propstream: PropStreamConfig,
    pub pipedrive: PipedriveConfig,
    pub database: DatabaseConfig,
    pub scheduler: SchedulerConfig,
    pub criteria: LeadCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropStreamConfig {
    pub api_key: String,
    pub max_results_per_run: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipedriveConfig {
    pub api_token: String,
    pub default_pipeline_id: Option<u64>,
    pub default_stage_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub cron_expression: String,
    pub max_leads_per_run: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            propstream: PropStreamConfig {
                api_key: String::new(),
                max_results_per_run: Some(1000),
            },
            pipedrive: PipedriveConfig {
                api_token: String::new(),
                default_pipeline_id: None,
                default_stage_id: None,
            },
            database: DatabaseConfig {
                url: "sqlite://propstream_pipedrive.db".to_string(),
            },
            scheduler: SchedulerConfig {
                enabled: true,
                cron_expression: "0 0 9 * * *".to_string(), // Daily at 9 AM
                max_leads_per_run: 100,
            },
            criteria: LeadCriteria::default(),
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn load(path: &PathBuf) -> Result<Self> {
        info!("Loading configuration from: {:?}", path);

        if !path.exists() {
            info!("Config file not found, creating default configuration");
            let config = Self::default();
            config.save(path)?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(path)
            .context("Failed to read config file")?;

        let config: AppConfig = serde_json::from_str(&content)
            .context("Failed to parse config file")?;

        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        info!("Saving configuration to: {:?}", path);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;

        std::fs::write(path, content)
            .context("Failed to write config file")?;

        info!("Configuration saved successfully");
        Ok(())
    }

    /// Load from environment variables
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let mut config = Self::default();

        if let Ok(api_key) = std::env::var("PROPSTREAM_API_KEY") {
            config.propstream.api_key = api_key;
        }

        if let Ok(api_token) = std::env::var("PIPEDRIVE_API_TOKEN") {
            config.pipedrive.api_token = api_token;
        }

        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database.url = db_url;
        }

        if let Ok(cron) = std::env::var("SCHEDULER_CRON") {
            config.scheduler.cron_expression = cron;
        }

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.propstream.api_key.is_empty() {
            anyhow::bail!("PropStream API key is required");
        }

        if self.pipedrive.api_token.is_empty() {
            anyhow::bail!("Pipedrive API token is required");
        }

        if self.database.url.is_empty() {
            anyhow::bail!("Database URL is required");
        }

        Ok(())
    }
}

/// Get default config file path
pub fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("propstream-pipedrive")
        .join("config.json")
}
