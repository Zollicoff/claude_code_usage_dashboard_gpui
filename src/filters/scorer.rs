use crate::models::Property;

/// Lead scoring engine to prioritize qualified leads
pub struct LeadScorer;

impl LeadScorer {
    /// Calculate a qualification score for a property (0-100)
    pub fn score(property: &Property) -> f64 {
        let mut score = 0.0;

        // Equity score (0-30 points)
        if let Some(equity) = property.calculate_equity_percent() {
            score += Self::equity_score(equity);
        }

        // Ownership duration score (0-20 points)
        score += Self::ownership_score(property.years_of_ownership());

        // Property condition score (0-15 points)
        score += Self::condition_score(property);

        // Motivation indicators score (0-25 points)
        score += Self::motivation_score(property);

        // Property value score (0-10 points)
        if let Some(value) = property.financial.estimated_value {
            score += Self::value_score(value);
        }

        score.min(100.0)
    }

    /// Score based on equity percentage (0-30 points)
    fn equity_score(equity_percent: f64) -> f64 {
        match equity_percent {
            e if e >= 80.0 => 30.0,
            e if e >= 60.0 => 25.0,
            e if e >= 50.0 => 20.0,
            e if e >= 40.0 => 15.0,
            e if e >= 30.0 => 10.0,
            _ => 5.0,
        }
    }

    /// Score based on ownership duration (0-20 points)
    fn ownership_score(years: f64) -> f64 {
        match years {
            y if y >= 20.0 => 20.0,
            y if y >= 15.0 => 17.0,
            y if y >= 10.0 => 14.0,
            y if y >= 7.0 => 11.0,
            y if y >= 5.0 => 8.0,
            _ => 3.0,
        }
    }

    /// Score based on property condition (0-15 points)
    fn condition_score(property: &Property) -> f64 {
        let mut score = 0.0;

        // Older properties may need work (good for investors)
        if let Some(years_old) = property.years_since_construction() {
            score += match years_old {
                y if y >= 50.0 => 8.0,
                y if y >= 30.0 => 7.0,
                y if y >= 20.0 => 6.0,
                y if y >= 10.0 => 5.0,
                _ => 3.0,
            };
        }

        // Vacant properties are opportunities
        if property.property_details.is_vacant {
            score += 7.0;
        } else {
            score += 2.0;
        }

        score
    }

    /// Score based on motivation indicators (0-25 points)
    fn motivation_score(property: &Property) -> f64 {
        let mut score = 0.0;

        // Absentee owner (high motivation)
        if property.owner.is_absentee {
            score += 10.0;
        }

        // Distressed property indicators
        if property.flags.is_distressed {
            score += 8.0;
        }

        if property.flags.is_pre_foreclosure {
            score += 5.0;
        }

        if property.flags.has_tax_lien {
            score += 4.0;
        }

        if property.flags.is_inherited || property.flags.is_probate {
            score += 5.0;
        }

        score.min(25.0)
    }

    /// Score based on property value (0-10 points)
    fn value_score(value: f64) -> f64 {
        match value {
            v if v >= 500_000.0 => 10.0,
            v if v >= 300_000.0 => 8.0,
            v if v >= 200_000.0 => 7.0,
            v if v >= 150_000.0 => 6.0,
            v if v >= 100_000.0 => 5.0,
            v if v >= 50_000.0 => 4.0,
            _ => 2.0,
        }
    }

    /// Get a human-readable score tier
    pub fn score_tier(score: f64) -> &'static str {
        match score {
            s if s >= 90.0 => "Exceptional",
            s if s >= 80.0 => "Excellent",
            s if s >= 70.0 => "Very Good",
            s if s >= 60.0 => "Good",
            s if s >= 50.0 => "Fair",
            _ => "Low Priority",
        }
    }
}
