use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use log::{info, error, debug};

use crate::models::{Property, PropertyAddress, OwnerInfo, PropertyDetails, PropertyType,
                     FinancialInfo, PropertyDates, PropertyFlags};

/// PropStream API client
pub struct PropStreamClient {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PropStreamSearchRequest {
    states: Vec<String>,
    counties: Vec<String>,
    property_types: Vec<String>,
    min_equity: Option<f64>,
    max_equity: Option<f64>,
    absentee_owner: Option<bool>,
    min_ownership_years: Option<f64>,
    page: u32,
    page_size: u32,
}

#[derive(Debug, Deserialize)]
struct PropStreamResponse {
    properties: Vec<PropStreamProperty>,
    total_count: u32,
    page: u32,
    page_size: u32,
}

#[derive(Debug, Deserialize)]
struct PropStreamProperty {
    property_id: String,
    address: PropStreamAddress,
    owner: PropStreamOwner,
    details: PropStreamDetails,
    financial: PropStreamFinancial,
    dates: PropStreamDates,
    flags: PropStreamFlags,
}

#[derive(Debug, Deserialize)]
struct PropStreamAddress {
    street_address: String,
    city: String,
    state: String,
    county: String,
    zip_code: String,
}

#[derive(Debug, Deserialize)]
struct PropStreamOwner {
    name: String,
    mailing_address: Option<String>,
    absentee_owner: bool,
    entity_type: Option<String>,
    years_owned: f64,
}

#[derive(Debug, Deserialize)]
struct PropStreamDetails {
    property_type: String,
    bedrooms: Option<i32>,
    bathrooms: Option<f32>,
    square_feet: Option<i32>,
    lot_size_sqft: Option<f64>,
    year_built: Option<i32>,
    vacant: bool,
}

#[derive(Debug, Deserialize)]
struct PropStreamFinancial {
    estimated_value: Option<f64>,
    assessed_value: Option<f64>,
    mortgage_amount: Option<f64>,
    equity_percent: Option<f64>,
    last_sale_price: Option<f64>,
    last_sale_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PropStreamDates {
    purchase_date: Option<String>,
    year_built: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct PropStreamFlags {
    distressed: bool,
    pre_foreclosure: bool,
    tax_lien: bool,
    inherited: bool,
    probate: bool,
}

impl PropStreamClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            api_key,
            base_url: "https://api.propstream.com/v1".to_string(),
        }
    }

    /// Fetch properties based on search criteria
    pub async fn fetch_properties(
        &self,
        states: Vec<String>,
        counties: Vec<String>,
        property_types: Vec<String>,
        min_equity: Option<f64>,
        max_equity: Option<f64>,
        absentee_owner: Option<bool>,
        min_ownership_years: Option<f64>,
        max_results: Option<u32>,
    ) -> Result<Vec<Property>> {
        info!("Fetching properties from PropStream API");
        debug!("Search criteria - States: {:?}, Counties: {:?}", states, counties);

        let mut all_properties = Vec::new();
        let page_size = 100;
        let mut page = 1;
        let max_pages = max_results.map(|m| (m / page_size) + 1).unwrap_or(10);

        loop {
            let request = PropStreamSearchRequest {
                states: states.clone(),
                counties: counties.clone(),
                property_types: property_types.clone(),
                min_equity,
                max_equity,
                absentee_owner,
                min_ownership_years,
                page,
                page_size,
            };

            let response = self.client
                .post(format!("{}/properties/search", self.base_url))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request)
                .send()
                .await
                .context("Failed to send PropStream API request")?;

            if !response.status().is_success() {
                error!("PropStream API error: {}", response.status());
                return Err(anyhow::anyhow!(
                    "PropStream API returned error: {}",
                    response.status()
                ));
            }

            let data: PropStreamResponse = response
                .json()
                .await
                .context("Failed to parse PropStream response")?;

            info!("Received {} properties from page {}", data.properties.len(), page);

            all_properties.extend(
                data.properties
                    .into_iter()
                    .map(|p| self.convert_property(p))
                    .collect::<Vec<_>>()
            );

            // Check if we should continue pagination
            if page >= max_pages || (page * page_size) >= data.total_count {
                break;
            }

            page += 1;
        }

        info!("Total properties fetched: {}", all_properties.len());
        Ok(all_properties)
    }

    /// Convert PropStream property to internal Property model
    fn convert_property(&self, prop: PropStreamProperty) -> Property {
        Property {
            id: prop.property_id,
            address: PropertyAddress {
                street: prop.address.street_address.clone(),
                city: prop.address.city.clone(),
                state: prop.address.state.clone(),
                county: prop.address.county.clone(),
                zip: prop.address.zip_code.clone(),
                full_address: format!(
                    "{}, {}, {} {}",
                    prop.address.street_address,
                    prop.address.city,
                    prop.address.state,
                    prop.address.zip_code
                ),
            },
            owner: OwnerInfo {
                name: prop.owner.name,
                mailing_address: prop.owner.mailing_address,
                is_absentee: prop.owner.absentee_owner,
                is_llc: prop.owner.entity_type.as_ref()
                    .map(|t| t.to_lowercase().contains("llc"))
                    .unwrap_or(false),
                is_trust: prop.owner.entity_type.as_ref()
                    .map(|t| t.to_lowercase().contains("trust"))
                    .unwrap_or(false),
                ownership_years: prop.owner.years_owned,
            },
            property_details: PropertyDetails {
                property_type: PropertyType::from_string(&prop.details.property_type),
                bedrooms: prop.details.bedrooms,
                bathrooms: prop.details.bathrooms,
                square_feet: prop.details.square_feet,
                lot_size: prop.details.lot_size_sqft,
                year_built: prop.details.year_built,
                is_vacant: prop.details.vacant,
            },
            financial: FinancialInfo {
                estimated_value: prop.financial.estimated_value,
                assessed_value: prop.financial.assessed_value,
                mortgage_balance: prop.financial.mortgage_amount,
                equity_percent: prop.financial.equity_percent,
                last_sale_amount: prop.financial.last_sale_price,
                last_sale_date: prop.financial.last_sale_date
                    .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                    .map(|d| d.with_timezone(&Utc)),
            },
            dates: PropertyDates {
                acquisition_date: prop.dates.purchase_date
                    .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
                    .map(|d| d.with_timezone(&Utc)),
                construction_date: prop.dates.year_built
                    .and_then(|year| chrono::NaiveDate::from_ymd_opt(year, 1, 1))
                    .map(|date| date.and_hms_opt(0, 0, 0).unwrap().and_utc()),
                last_updated: Utc::now(),
            },
            flags: PropertyFlags {
                is_distressed: prop.flags.distressed,
                is_pre_foreclosure: prop.flags.pre_foreclosure,
                has_tax_lien: prop.flags.tax_lien,
                is_inherited: prop.flags.inherited,
                is_probate: prop.flags.probate,
            },
        }
    }

    /// Test API connection
    pub async fn test_connection(&self) -> Result<bool> {
        info!("Testing PropStream API connection");

        let response = self.client
            .get(format!("{}/health", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
