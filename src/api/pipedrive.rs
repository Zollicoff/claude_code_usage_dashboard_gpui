use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use log::{info, error, debug};

use crate::models::{PipedriveLead, QualifiedLead};

/// Pipedrive API client
pub struct PipedriveClient {
    client: Client,
    api_token: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct PipedriveLeadRequest {
    title: String,
    person_id: Option<u64>,
    organization_id: Option<u64>,
    value: Option<f64>,
    currency: String,
    #[serde(flatten)]
    custom_fields: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct PipedriveLeadResponse {
    success: bool,
    data: Option<PipedriveLeadData>,
}

#[derive(Debug, Deserialize)]
struct PipedriveLeadData {
    id: u64,
    title: String,
}

#[derive(Debug, Serialize)]
struct PipedrivePersonRequest {
    name: String,
    email: Option<Vec<String>>,
    phone: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PipedrivePersonResponse {
    success: bool,
    data: Option<PipedrivePersonData>,
}

#[derive(Debug, Deserialize)]
struct PipedrivePersonData {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct PipedriveSearchResponse {
    success: bool,
    data: Option<PipedriveSearchData>,
}

#[derive(Debug, Deserialize)]
struct PipedriveSearchData {
    items: Vec<PipedriveSearchItem>,
}

#[derive(Debug, Deserialize)]
struct PipedriveSearchItem {
    item: PipedriveSearchItemData,
}

#[derive(Debug, Deserialize)]
struct PipedriveSearchItemData {
    id: u64,
}

impl PipedriveClient {
    pub fn new(api_token: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            api_token,
            base_url: "https://api.pipedrive.com/v1".to_string(),
        }
    }

    /// Create a lead in Pipedrive
    pub async fn create_lead(&self, lead: &QualifiedLead) -> Result<String> {
        let pipedrive_lead: PipedriveLead = lead.into();

        info!("Creating lead in Pipedrive: {}", pipedrive_lead.title);
        debug!("Lead details: {:?}", pipedrive_lead);

        // First, create or find the person
        let person_id = self.create_or_find_person(&pipedrive_lead.person_name).await?;

        // Create organization if needed
        let organization_id = if let Some(org_name) = &pipedrive_lead.organization_name {
            Some(self.create_or_find_organization(org_name).await?)
        } else {
            None
        };

        // Convert custom fields to JSON
        let custom_fields = serde_json::to_value(&pipedrive_lead.custom_fields)?;

        let request = PipedriveLeadRequest {
            title: pipedrive_lead.title.clone(),
            person_id: Some(person_id),
            organization_id,
            value: pipedrive_lead.value,
            currency: pipedrive_lead.currency.clone(),
            custom_fields,
        };

        let response = self.client
            .post(format!("{}/deals", self.base_url))
            .query(&[("api_token", &self.api_token)])
            .json(&request)
            .send()
            .await
            .context("Failed to send Pipedrive API request")?;

        if !response.status().is_success() {
            error!("Pipedrive API error: {}", response.status());
            let error_text = response.text().await.unwrap_or_default();
            error!("Error details: {}", error_text);
            return Err(anyhow::anyhow!(
                "Pipedrive API returned error: {}",
                error_text
            ));
        }

        let result: PipedriveLeadResponse = response
            .json()
            .await
            .context("Failed to parse Pipedrive response")?;

        if !result.success {
            return Err(anyhow::anyhow!("Pipedrive lead creation failed"));
        }

        let lead_id = result.data
            .ok_or_else(|| anyhow::anyhow!("No data in Pipedrive response"))?
            .id
            .to_string();

        info!("Successfully created lead in Pipedrive with ID: {}", lead_id);
        Ok(lead_id)
    }

    /// Create or find a person in Pipedrive
    async fn create_or_find_person(&self, name: &str) -> Result<u64> {
        // First, search for existing person
        let search_response = self.client
            .get(format!("{}/persons/search", self.base_url))
            .query(&[
                ("term", name),
                ("api_token", &self.api_token),
            ])
            .send()
            .await?;

        if search_response.status().is_success() {
            let search_result: PipedriveSearchResponse = search_response.json().await?;
            if let Some(data) = search_result.data {
                if !data.items.is_empty() {
                    debug!("Found existing person: {}", name);
                    return Ok(data.items[0].item.id);
                }
            }
        }

        // Person not found, create new
        debug!("Creating new person: {}", name);
        let request = PipedrivePersonRequest {
            name: name.to_string(),
            email: None,
            phone: None,
        };

        let response = self.client
            .post(format!("{}/persons", self.base_url))
            .query(&[("api_token", &self.api_token)])
            .json(&request)
            .send()
            .await?;

        let result: PipedrivePersonResponse = response.json().await?;

        result.data
            .ok_or_else(|| anyhow::anyhow!("Failed to create person"))?
            .id
            .into()
    }

    /// Create or find an organization in Pipedrive
    async fn create_or_find_organization(&self, name: &str) -> Result<u64> {
        // Search for existing organization
        let search_response = self.client
            .get(format!("{}/organizations/search", self.base_url))
            .query(&[
                ("term", name),
                ("api_token", &self.api_token),
            ])
            .send()
            .await?;

        if search_response.status().is_success() {
            let search_result: PipedriveSearchResponse = search_response.json().await?;
            if let Some(data) = search_result.data {
                if !data.items.is_empty() {
                    debug!("Found existing organization: {}", name);
                    return Ok(data.items[0].item.id);
                }
            }
        }

        // Organization not found, create new
        debug!("Creating new organization: {}", name);
        let request = serde_json::json!({
            "name": name
        });

        let response = self.client
            .post(format!("{}/organizations", self.base_url))
            .query(&[("api_token", &self.api_token)])
            .json(&request)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        result["data"]["id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Failed to create organization"))
    }

    /// Check if a lead already exists by address
    pub async fn check_duplicate(&self, address: &str) -> Result<bool> {
        debug!("Checking for duplicate lead: {}", address);

        let response = self.client
            .get(format!("{}/deals/search", self.base_url))
            .query(&[
                ("term", address),
                ("api_token", &self.api_token),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let result: PipedriveSearchResponse = response.json().await?;

        Ok(result.data.map(|d| !d.items.is_empty()).unwrap_or(false))
    }

    /// Test API connection
    pub async fn test_connection(&self) -> Result<bool> {
        info!("Testing Pipedrive API connection");

        let response = self.client
            .get(format!("{}/users", self.base_url))
            .query(&[("api_token", &self.api_token)])
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
