use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a property from PropStream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: String,
    pub address: PropertyAddress,
    pub owner: OwnerInfo,
    pub property_details: PropertyDetails,
    pub financial: FinancialInfo,
    pub dates: PropertyDates,
    pub flags: PropertyFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyAddress {
    pub street: String,
    pub city: String,
    pub state: String,
    pub county: String,
    pub zip: String,
    pub full_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerInfo {
    pub name: String,
    pub mailing_address: Option<String>,
    pub is_absentee: bool,
    pub is_llc: bool,
    pub is_trust: bool,
    pub ownership_years: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDetails {
    pub property_type: PropertyType,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<f32>,
    pub square_feet: Option<i32>,
    pub lot_size: Option<f64>,
    pub year_built: Option<i32>,
    pub is_vacant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyType {
    SingleFamily,
    MultiFamily,
    VacantLand,
    Commercial,
    Other(String),
}

impl PropertyType {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "single family" | "sfr" => PropertyType::SingleFamily,
            "multi family" | "multifamily" => PropertyType::MultiFamily,
            "vacant land" | "land" => PropertyType::VacantLand,
            "commercial" => PropertyType::Commercial,
            _ => PropertyType::Other(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialInfo {
    pub estimated_value: Option<f64>,
    pub assessed_value: Option<f64>,
    pub mortgage_balance: Option<f64>,
    pub equity_percent: Option<f64>,
    pub last_sale_amount: Option<f64>,
    pub last_sale_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDates {
    pub acquisition_date: Option<DateTime<Utc>>,
    pub construction_date: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyFlags {
    pub is_distressed: bool,
    pub is_pre_foreclosure: bool,
    pub has_tax_lien: bool,
    pub is_inherited: bool,
    pub is_probate: bool,
}

impl Property {
    /// Calculate the current equity percentage
    pub fn calculate_equity_percent(&self) -> Option<f64> {
        if let (Some(value), Some(mortgage)) =
            (self.financial.estimated_value, self.financial.mortgage_balance) {
            if value > 0.0 {
                return Some(((value - mortgage) / value) * 100.0);
            }
        }
        self.financial.equity_percent
    }

    /// Get the years since construction
    pub fn years_since_construction(&self) -> Option<f64> {
        self.dates.construction_date.map(|date| {
            let now = Utc::now();
            let duration = now.signed_duration_since(date);
            duration.num_days() as f64 / 365.25
        })
    }

    /// Get the years of ownership
    pub fn years_of_ownership(&self) -> f64 {
        self.owner.ownership_years
    }
}
