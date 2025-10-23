use serde::{Deserialize, Serialize};
use super::property::PropertyType;

/// Filter criteria for qualifying leads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadCriteria {
    pub geographic: GeographicFilters,
    pub property_types: PropertyTypeFilters,
    pub owner: OwnerFilters,
    pub financial: FinancialFilters,
    pub timing: TimingFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicFilters {
    pub states: Vec<String>,
    pub counties: Vec<String>,
    pub exclude_cities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyTypeFilters {
    pub included_types: Vec<PropertyType>,
    pub allow_distressed: bool,
    pub allow_vacant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerFilters {
    pub require_absentee: bool,
    pub exclude_llc: bool,
    pub exclude_trust: bool,
    pub min_ownership_years: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialFilters {
    pub min_equity_percent: f64,
    pub max_equity_percent: f64,
    pub min_property_value: Option<f64>,
    pub max_property_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingFilters {
    pub min_construction_years: Option<f64>,
    pub max_days_since_last_sale: Option<i64>,
}

impl Default for LeadCriteria {
    fn default() -> Self {
        Self {
            geographic: GeographicFilters {
                states: vec!["NJ".to_string(), "PA".to_string(), "DE".to_string()],
                counties: vec![
                    "Camden".to_string(),
                    "Burlington".to_string(),
                    "Gloucester".to_string(),
                    "Atlantic".to_string(),
                    "Cape May".to_string(),
                    "Philadelphia".to_string(),
                    "Delaware".to_string(),
                    "Bucks".to_string(),
                ],
                exclude_cities: vec![],
            },
            property_types: PropertyTypeFilters {
                included_types: vec![
                    PropertyType::SingleFamily,
                    PropertyType::MultiFamily,
                    PropertyType::VacantLand,
                ],
                allow_distressed: true,
                allow_vacant: true,
            },
            owner: OwnerFilters {
                require_absentee: true,
                exclude_llc: true,
                exclude_trust: true,
                min_ownership_years: 5.0,
            },
            financial: FinancialFilters {
                min_equity_percent: 30.0,
                max_equity_percent: 100.0,
                min_property_value: None,
                max_property_value: None,
            },
            timing: TimingFilters {
                min_construction_years: Some(10.0),
                max_days_since_last_sale: None,
            },
        }
    }
}
