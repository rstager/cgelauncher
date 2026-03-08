use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingEstimate {
    pub spot_hourly: f64,
    pub ondemand_hourly: f64,
    pub currency: String,
    pub breakdown: Vec<PricingLineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PricingLineItem {
    pub label: String,
    pub spot_cost: f64,
    pub ondemand_cost: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_serializes_roundtrip() {
        let estimate = PricingEstimate {
            spot_hourly: 1.42,
            ondemand_hourly: 4.86,
            currency: "USD".into(),
            breakdown: vec![
                PricingLineItem {
                    label: "8x vCPU".into(),
                    spot_cost: 0.08,
                    ondemand_cost: 0.25,
                },
                PricingLineItem {
                    label: "30 GB Memory".into(),
                    spot_cost: 0.04,
                    ondemand_cost: 0.13,
                },
            ],
        };
        let json = serde_json::to_string(&estimate).unwrap();
        let parsed: PricingEstimate = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.spot_hourly, 1.42);
        assert_eq!(parsed.breakdown.len(), 2);
    }
}
