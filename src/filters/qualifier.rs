use log::{debug, info};
use crate::models::{Property, LeadCriteria, PropertyType};

/// Lead qualification engine
pub struct LeadQualifier {
    criteria: LeadCriteria,
}

impl LeadQualifier {
    pub fn new(criteria: LeadCriteria) -> Self {
        Self { criteria }
    }

    /// Qualify a property against all criteria
    pub fn qualify(&self, property: &Property) -> QualificationResult {
        let mut result = QualificationResult::default();

        // Geographic filters
        if !self.check_geographic(property) {
            result.passed = false;
            result.reasons.push("Geographic criteria not met".to_string());
            return result;
        }

        // Property type filters
        if !self.check_property_type(property) {
            result.passed = false;
            result.reasons.push("Property type not in allowed list".to_string());
            return result;
        }

        // Owner filters
        if !self.check_owner_criteria(property) {
            result.passed = false;
            result.reasons.push("Owner criteria not met".to_string());
            return result;
        }

        // Financial filters
        if !self.check_financial_criteria(property) {
            result.passed = false;
            result.reasons.push("Financial criteria not met".to_string());
            return result;
        }

        // Timing filters
        if !self.check_timing_criteria(property) {
            result.passed = false;
            result.reasons.push("Timing criteria not met".to_string());
            return result;
        }

        result.passed = true;
        result.reasons.push("All criteria met".to_string());
        result
    }

    /// Check geographic criteria
    fn check_geographic(&self, property: &Property) -> bool {
        let geo = &self.criteria.geographic;

        // Check state
        if !geo.states.is_empty() && !geo.states.contains(&property.address.state) {
            debug!("Property state {} not in allowed states: {:?}",
                   property.address.state, geo.states);
            return false;
        }

        // Check county
        if !geo.counties.is_empty() && !geo.counties.contains(&property.address.county) {
            debug!("Property county {} not in allowed counties: {:?}",
                   property.address.county, geo.counties);
            return false;
        }

        // Check excluded cities
        if geo.exclude_cities.contains(&property.address.city) {
            debug!("Property city {} is excluded", property.address.city);
            return false;
        }

        true
    }

    /// Check property type criteria
    fn check_property_type(&self, property: &Property) -> bool {
        let type_filters = &self.criteria.property_types;

        // Check if property type is in allowed list
        let type_match = type_filters.included_types.iter().any(|allowed_type| {
            match (allowed_type, &property.property_details.property_type) {
                (PropertyType::SingleFamily, PropertyType::SingleFamily) => true,
                (PropertyType::MultiFamily, PropertyType::MultiFamily) => true,
                (PropertyType::VacantLand, PropertyType::VacantLand) => true,
                (PropertyType::Commercial, PropertyType::Commercial) => true,
                _ => false,
            }
        });

        if !type_match {
            debug!("Property type {:?} not in allowed types",
                   property.property_details.property_type);
            return false;
        }

        // Check distressed property flag
        if property.flags.is_distressed && !type_filters.allow_distressed {
            debug!("Distressed properties not allowed");
            return false;
        }

        // Check vacant property flag
        if property.property_details.is_vacant && !type_filters.allow_vacant {
            debug!("Vacant properties not allowed");
            return false;
        }

        true
    }

    /// Check owner criteria
    fn check_owner_criteria(&self, property: &Property) -> bool {
        let owner_filters = &self.criteria.owner;

        // Check absentee owner requirement
        if owner_filters.require_absentee && !property.owner.is_absentee {
            debug!("Property owner is not absentee");
            return false;
        }

        // Check LLC exclusion
        if owner_filters.exclude_llc && property.owner.is_llc {
            debug!("LLC ownership excluded");
            return false;
        }

        // Check trust exclusion
        if owner_filters.exclude_trust && property.owner.is_trust {
            debug!("Trust ownership excluded");
            return false;
        }

        // Check minimum ownership years
        if property.owner.ownership_years < owner_filters.min_ownership_years {
            debug!("Ownership years {} below minimum {}",
                   property.owner.ownership_years,
                   owner_filters.min_ownership_years);
            return false;
        }

        true
    }

    /// Check financial criteria
    fn check_financial_criteria(&self, property: &Property) -> bool {
        let financial = &self.criteria.financial;

        // Calculate equity percentage
        let equity_percent = match property.calculate_equity_percent() {
            Some(eq) => eq,
            None => {
                debug!("Cannot calculate equity percentage");
                return false;
            }
        };

        // Check equity range
        if equity_percent < financial.min_equity_percent
            || equity_percent > financial.max_equity_percent {
            debug!("Equity {}% outside range {}-{}%",
                   equity_percent,
                   financial.min_equity_percent,
                   financial.max_equity_percent);
            return false;
        }

        // Check property value range if specified
        if let Some(min_value) = financial.min_property_value {
            if let Some(value) = property.financial.estimated_value {
                if value < min_value {
                    debug!("Property value ${} below minimum ${}", value, min_value);
                    return false;
                }
            } else {
                debug!("Property value not available");
                return false;
            }
        }

        if let Some(max_value) = financial.max_property_value {
            if let Some(value) = property.financial.estimated_value {
                if value > max_value {
                    debug!("Property value ${} above maximum ${}", value, max_value);
                    return false;
                }
            }
        }

        true
    }

    /// Check timing criteria
    fn check_timing_criteria(&self, property: &Property) -> bool {
        let timing = &self.criteria.timing;

        // Check minimum construction years
        if let Some(min_years) = timing.min_construction_years {
            match property.years_since_construction() {
                Some(years) => {
                    if years < min_years {
                        debug!("Construction years {} below minimum {}", years, min_years);
                        return false;
                    }
                }
                None => {
                    debug!("Construction date not available");
                    return false;
                }
            }
        }

        // Check days since last sale
        if let Some(max_days) = timing.max_days_since_last_sale {
            if let Some(last_sale) = property.financial.last_sale_date {
                let days_since_sale = (chrono::Utc::now() - last_sale).num_days();
                if days_since_sale > max_days {
                    debug!("Days since last sale {} exceeds maximum {}",
                           days_since_sale, max_days);
                    return false;
                }
            }
        }

        true
    }

    /// Batch qualify multiple properties
    pub fn qualify_batch(&self, properties: Vec<Property>) -> Vec<(Property, QualificationResult)> {
        info!("Qualifying batch of {} properties", properties.len());

        let results: Vec<_> = properties
            .into_iter()
            .map(|prop| {
                let result = self.qualify(&prop);
                (prop, result)
            })
            .collect();

        let qualified_count = results.iter().filter(|(_, r)| r.passed).count();
        info!("Qualified {}/{} properties", qualified_count, results.len());

        results
    }
}

#[derive(Debug, Default)]
pub struct QualificationResult {
    pub passed: bool,
    pub reasons: Vec<String>,
}
