use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::property::Property;

/// Represents a qualified lead ready for Pipedrive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualifiedLead {
    pub id: String,
    pub property: Property,
    pub qualification_score: f64,
    pub created_at: DateTime<Utc>,
    pub pipedrive_id: Option<String>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Pending,
    Synced,
    Failed(String),
    Duplicate,
}

/// Pipedrive lead structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipedriveLead {
    pub title: String,
    pub person_name: String,
    pub organization_name: Option<String>,
    pub value: Option<f64>,
    pub currency: String,
    pub custom_fields: PipedriveCustomFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipedriveCustomFields {
    pub property_address: String,
    pub property_type: String,
    pub equity_percent: Option<f64>,
    pub ownership_years: f64,
    pub is_absentee: bool,
    pub estimated_value: Option<f64>,
    pub year_built: Option<i32>,
    pub construction_years: Option<f64>,
    pub county: String,
    pub state: String,
    pub is_distressed: bool,
    pub qualification_score: f64,
}

impl From<&QualifiedLead> for PipedriveLead {
    fn from(lead: &QualifiedLead) -> Self {
        let prop = &lead.property;

        PipedriveLead {
            title: format!("{} - {}", prop.address.full_address, prop.owner.name),
            person_name: prop.owner.name.clone(),
            organization_name: if prop.owner.is_llc || prop.owner.is_trust {
                Some(prop.owner.name.clone())
            } else {
                None
            },
            value: prop.financial.estimated_value,
            currency: "USD".to_string(),
            custom_fields: PipedriveCustomFields {
                property_address: prop.address.full_address.clone(),
                property_type: format!("{:?}", prop.property_details.property_type),
                equity_percent: prop.calculate_equity_percent(),
                ownership_years: prop.owner.ownership_years,
                is_absentee: prop.owner.is_absentee,
                estimated_value: prop.financial.estimated_value,
                year_built: prop.property_details.year_built,
                construction_years: prop.years_since_construction(),
                county: prop.address.county.clone(),
                state: prop.address.state.clone(),
                is_distressed: prop.flags.is_distressed,
                qualification_score: lead.qualification_score,
            },
        }
    }
}

impl QualifiedLead {
    pub fn new(property: Property, qualification_score: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            property,
            qualification_score,
            created_at: Utc::now(),
            pipedrive_id: None,
            sync_status: SyncStatus::Pending,
        }
    }
}
