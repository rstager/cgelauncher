use crate::gcloud::pricing_fetch::{gcloud_gpu_to_page_name, SpotPricingCache};
use crate::models::machine::MachineConfig;
use crate::models::pricing::{PricingEstimate, PricingLineItem};

// ── Static fallback prices (used when cache is unavailable) ──

const N1_VCPU_SPOT: f64 = 0.00700;
const N1_VCPU_ONDEMAND: f64 = 0.033174;

const N1_MEM_SPOT: f64 = 0.000936;
const N1_MEM_ONDEMAND: f64 = 0.004446;

struct GpuPrice {
    spot: f64,
    ondemand: f64,
}

fn static_gpu_price(gpu_type: &str) -> Option<GpuPrice> {
    match gpu_type {
        "nvidia-tesla-t4" => Some(GpuPrice {
            spot: 0.12,
            ondemand: 0.35,
        }),
        "nvidia-tesla-v100" => Some(GpuPrice {
            spot: 0.74,
            ondemand: 2.48,
        }),
        "nvidia-tesla-p100" => Some(GpuPrice {
            spot: 0.43,
            ondemand: 1.46,
        }),
        "nvidia-tesla-a100" => Some(GpuPrice {
            spot: 0.88,
            ondemand: 2.934,
        }),
        "nvidia-l4" => Some(GpuPrice {
            spot: 0.22,
            ondemand: 0.72,
        }),
        _ => None,
    }
}

fn static_a2_price(machine_type: &str) -> Option<(f64, f64)> {
    match machine_type {
        "a2-highgpu-1g" => Some((1.10, 3.67)),
        "a2-highgpu-2g" => Some((2.20, 7.35)),
        "a2-highgpu-4g" => Some((4.41, 14.69)),
        "a2-highgpu-8g" => Some((8.82, 29.39)),
        "a2-megagpu-16g" => Some((16.72, 55.74)),
        _ => None,
    }
}

fn static_g2_price(machine_type: &str) -> Option<(f64, f64)> {
    match machine_type {
        // (spot, on-demand) — approximate us-central1 pricing
        "g2-standard-4" => Some((0.23, 0.77)),
        "g2-standard-8" => Some((0.46, 1.53)),
        "g2-standard-12" => Some((0.56, 1.84)),
        "g2-standard-16" => Some((0.92, 3.07)),
        "g2-standard-24" => Some((1.38, 4.60)),
        "g2-standard-32" => Some((1.84, 6.14)),
        "g2-standard-48" => Some((2.75, 9.21)),
        "g2-standard-96" => Some((5.50, 18.41)),
        _ => None,
    }
}

// ── Machine type helpers ──

fn vcpus_from_machine_type(mt: &str) -> Option<u32> {
    let suffix = mt
        .strip_prefix("n1-standard-")
        .or_else(|| mt.strip_prefix("n1-highmem-"))
        .or_else(|| mt.strip_prefix("n1-highcpu-"))?;
    suffix.parse().ok()
}

fn memory_gb_from_machine_type(mt: &str) -> Option<f64> {
    if let Some(rest) = mt.strip_prefix("n1-standard-") {
        rest.parse::<f64>().ok().map(|v| v * 3.75)
    } else if let Some(rest) = mt.strip_prefix("n1-highmem-") {
        rest.parse::<f64>().ok().map(|v| v * 6.5)
    } else if let Some(rest) = mt.strip_prefix("n1-highcpu-") {
        rest.parse::<f64>().ok().map(|v| v * 0.9)
    } else {
        None
    }
}

// ── On-demand/spot ratio for deriving on-demand from cached spot ──

const ONDEMAND_SPOT_RATIO_N1: f64 = 4.75;
const ONDEMAND_SPOT_RATIO_GPU: f64 = 3.0;
const ONDEMAND_SPOT_RATIO_A2: f64 = 2.0;

// ── Public API ──

pub fn estimate_pricing(
    config: &MachineConfig,
    cache: Option<&SpotPricingCache>,
) -> Result<PricingEstimate, String> {
    // A2/A3 and other all-in GPU machine types
    if config.machine_type.starts_with("a2-")
        || config.machine_type.starts_with("a3-")
        || config.machine_type.starts_with("a4-")
        || config.machine_type.starts_with("g2-")
        || config.machine_type.starts_with("g4-")
    {
        return estimate_allin_machine(config, cache);
    }

    // N1 series: itemized vCPU + memory + optional GPU
    estimate_n1(config, cache)
}

fn estimate_allin_machine(
    config: &MachineConfig,
    cache: Option<&SpotPricingCache>,
) -> Result<PricingEstimate, String> {
    // Try cached price first
    if let Some(cached) = cache {
        if let Some(&spot) = cached.machine_prices.get(&config.machine_type) {
            let ondemand = spot * ONDEMAND_SPOT_RATIO_A2;
            return Ok(PricingEstimate {
                spot_hourly: round2(spot),
                ondemand_hourly: round2(ondemand),
                currency: "USD".into(),
                breakdown: vec![PricingLineItem {
                    label: config.machine_type.clone(),
                    spot_cost: round2(spot),
                    ondemand_cost: round2(ondemand),
                }],
            });
        }
    }

    // Static fallback (A2 and G2)
    let static_price = static_a2_price(&config.machine_type)
        .or_else(|| static_g2_price(&config.machine_type));
    if let Some((spot, ondemand)) = static_price {
        return Ok(PricingEstimate {
            spot_hourly: spot,
            ondemand_hourly: ondemand,
            currency: "USD".into(),
            breakdown: vec![PricingLineItem {
                label: config.machine_type.clone(),
                spot_cost: spot,
                ondemand_cost: ondemand,
            }],
        });
    }

    Err(format!(
        "Pricing unavailable for machine type: {}",
        config.machine_type
    ))
}

fn estimate_n1(
    config: &MachineConfig,
    cache: Option<&SpotPricingCache>,
) -> Result<PricingEstimate, String> {
    // If we have a cached all-in spot price for this N1 type, use it directly
    if let Some(cached) = cache {
        if let Some(&total_spot) = cached.machine_prices.get(&config.machine_type) {
            return estimate_n1_from_cached_spot(config, cached, total_spot);
        }
    }

    // Fall back to static component-based pricing
    estimate_n1_static(config)
}

fn estimate_n1_from_cached_spot(
    config: &MachineConfig,
    cache: &SpotPricingCache,
    machine_spot: f64,
) -> Result<PricingEstimate, String> {
    let machine_ondemand = machine_spot * ONDEMAND_SPOT_RATIO_N1;

    let mut breakdown = Vec::new();
    breakdown.push(PricingLineItem {
        label: format!("{} (compute)", config.machine_type),
        spot_cost: round2(machine_spot),
        ondemand_cost: round2(machine_ondemand),
    });

    let mut total_spot = machine_spot;
    let mut total_ondemand = machine_ondemand;

    if let (Some(gpu_type), Some(count)) = (&config.gpu_type, config.gpu_count) {
        let (gpu_spot, gpu_ondemand) = resolve_gpu_price(gpu_type, cache)?;
        let gpu_spot_total = gpu_spot * count as f64;
        let gpu_ondemand_total = gpu_ondemand * count as f64;
        total_spot += gpu_spot_total;
        total_ondemand += gpu_ondemand_total;

        let display_name = gpu_type
            .replace("nvidia-tesla-", "NVIDIA Tesla ")
            .replace("nvidia-", "NVIDIA ");
        breakdown.push(PricingLineItem {
            label: format!("{}x {}", count, display_name),
            spot_cost: round2(gpu_spot_total),
            ondemand_cost: round2(gpu_ondemand_total),
        });
    }

    Ok(PricingEstimate {
        spot_hourly: round2(total_spot),
        ondemand_hourly: round2(total_ondemand),
        currency: "USD".into(),
        breakdown,
    })
}

fn estimate_n1_static(config: &MachineConfig) -> Result<PricingEstimate, String> {
    let vcpus = vcpus_from_machine_type(&config.machine_type).ok_or_else(|| {
        format!(
            "Pricing unavailable for machine type: {}",
            config.machine_type
        )
    })?;

    let mem_gb = memory_gb_from_machine_type(&config.machine_type).ok_or_else(|| {
        format!(
            "Cannot determine memory for machine type: {}",
            config.machine_type
        )
    })?;

    let mut breakdown = Vec::new();

    let vcpu_spot = vcpus as f64 * N1_VCPU_SPOT;
    let vcpu_ondemand = vcpus as f64 * N1_VCPU_ONDEMAND;
    breakdown.push(PricingLineItem {
        label: format!("{}x vCPU ({})", vcpus, config.machine_type),
        spot_cost: round2(vcpu_spot),
        ondemand_cost: round2(vcpu_ondemand),
    });

    let mem_spot = mem_gb * N1_MEM_SPOT;
    let mem_ondemand = mem_gb * N1_MEM_ONDEMAND;
    breakdown.push(PricingLineItem {
        label: format!("{} GB Memory", mem_gb),
        spot_cost: round2(mem_spot),
        ondemand_cost: round2(mem_ondemand),
    });

    let mut total_spot = vcpu_spot + mem_spot;
    let mut total_ondemand = vcpu_ondemand + mem_ondemand;

    if let (Some(gpu_type), Some(count)) = (&config.gpu_type, config.gpu_count) {
        let price = static_gpu_price(gpu_type)
            .ok_or_else(|| format!("Pricing unavailable for GPU type: {gpu_type}"))?;
        let gpu_spot = price.spot * count as f64;
        let gpu_ondemand = price.ondemand * count as f64;
        total_spot += gpu_spot;
        total_ondemand += gpu_ondemand;

        let display_name = gpu_type
            .replace("nvidia-tesla-", "NVIDIA Tesla ")
            .replace("nvidia-", "NVIDIA ");
        breakdown.push(PricingLineItem {
            label: format!("{}x {}", count, display_name),
            spot_cost: round2(gpu_spot),
            ondemand_cost: round2(gpu_ondemand),
        });
    }

    Ok(PricingEstimate {
        spot_hourly: round2(total_spot),
        ondemand_hourly: round2(total_ondemand),
        currency: "USD".into(),
        breakdown,
    })
}

fn resolve_gpu_price(
    gpu_type: &str,
    cache: &SpotPricingCache,
) -> Result<(f64, f64), String> {
    if let Some(page_name) = gcloud_gpu_to_page_name(gpu_type) {
        if let Some(&spot) = cache.gpu_prices.get(page_name) {
            return Ok((spot, spot * ONDEMAND_SPOT_RATIO_GPU));
        }
    }
    let price = static_gpu_price(gpu_type)
        .ok_or_else(|| format!("Pricing unavailable for GPU type: {gpu_type}"))?;
    Ok((price.spot, price.ondemand))
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn n1_standard_8_no_gpu_static() {
        let config = MachineConfig {
            machine_type: "n1-standard-8".into(),
            gpu_type: None,
            gpu_count: None,
            spot: true,
        };
        let est = estimate_pricing(&config, None).unwrap();
        assert!(est.spot_hourly > 0.0);
        assert!(est.ondemand_hourly > est.spot_hourly);
        assert_eq!(est.breakdown.len(), 2);
        assert_eq!(est.currency, "USD");
    }

    #[test]
    fn n1_standard_8_with_t4_static() {
        let config = MachineConfig {
            machine_type: "n1-standard-8".into(),
            gpu_type: Some("nvidia-tesla-t4".into()),
            gpu_count: Some(4),
            spot: true,
        };
        let est = estimate_pricing(&config, None).unwrap();
        assert_eq!(est.breakdown.len(), 3);
        let gpu_line = &est.breakdown[2];
        assert!(gpu_line.spot_cost > est.breakdown[0].spot_cost);
    }

    #[test]
    fn a2_highgpu_pricing_static() {
        let config = MachineConfig {
            machine_type: "a2-highgpu-1g".into(),
            gpu_type: None,
            gpu_count: None,
            spot: false,
        };
        let est = estimate_pricing(&config, None).unwrap();
        assert_eq!(est.breakdown.len(), 1);
        assert_eq!(est.breakdown[0].label, "a2-highgpu-1g");
    }

    #[test]
    fn unknown_machine_type_errors() {
        let config = MachineConfig {
            machine_type: "e2-medium".into(),
            gpu_type: None,
            gpu_count: None,
            spot: true,
        };
        let result = estimate_pricing(&config, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Pricing unavailable"));
    }

    #[test]
    fn unknown_gpu_type_errors() {
        let config = MachineConfig {
            machine_type: "n1-standard-4".into(),
            gpu_type: Some("nvidia-tesla-h100".into()),
            gpu_count: Some(1),
            spot: true,
        };
        let result = estimate_pricing(&config, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("GPU type"));
    }

    #[test]
    fn vcpus_from_known_types() {
        assert_eq!(vcpus_from_machine_type("n1-standard-8"), Some(8));
        assert_eq!(vcpus_from_machine_type("n1-highmem-16"), Some(16));
        assert_eq!(vcpus_from_machine_type("n1-highcpu-4"), Some(4));
    }

    #[test]
    fn memory_from_known_types() {
        assert_eq!(memory_gb_from_machine_type("n1-standard-8"), Some(30.0));
        assert_eq!(memory_gb_from_machine_type("n1-highmem-8"), Some(52.0));
    }

    #[test]
    fn spot_is_cheaper_than_ondemand_static() {
        let config = MachineConfig {
            machine_type: "n1-standard-16".into(),
            gpu_type: Some("nvidia-tesla-v100".into()),
            gpu_count: Some(4),
            spot: true,
        };
        let est = estimate_pricing(&config, None).unwrap();
        assert!(est.spot_hourly < est.ondemand_hourly);
        for item in &est.breakdown {
            assert!(item.spot_cost <= item.ondemand_cost);
        }
    }

    #[test]
    fn cached_n1_uses_live_price() {
        let mut machine_prices = HashMap::new();
        machine_prices.insert("n1-standard-8".to_string(), 0.10446);
        let mut gpu_prices = HashMap::new();
        gpu_prices.insert("T4".to_string(), 0.175);

        let cache = SpotPricingCache {
            fetched_at: chrono::Utc::now(),
            machine_prices,
            gpu_prices,
        };

        let config = MachineConfig {
            machine_type: "n1-standard-8".into(),
            gpu_type: Some("nvidia-tesla-t4".into()),
            gpu_count: Some(4),
            spot: true,
        };
        let est = estimate_pricing(&config, Some(&cache)).unwrap();

        // Machine compute should be the cached spot price
        assert_eq!(est.breakdown[0].spot_cost, round2(0.10446));
        // GPU should use cached T4 price: 4 * 0.175 = 0.70
        assert_eq!(est.breakdown[1].spot_cost, 0.70);
    }

    #[test]
    fn cached_a2_uses_live_price() {
        let mut machine_prices = HashMap::new();
        machine_prices.insert("a2-highgpu-1g".to_string(), 1.80385);

        let cache = SpotPricingCache {
            fetched_at: chrono::Utc::now(),
            machine_prices,
            gpu_prices: HashMap::new(),
        };

        let config = MachineConfig {
            machine_type: "a2-highgpu-1g".into(),
            gpu_type: None,
            gpu_count: None,
            spot: false,
        };
        let est = estimate_pricing(&config, Some(&cache)).unwrap();
        assert_eq!(est.breakdown[0].spot_cost, round2(1.80385));
    }

    #[test]
    fn cache_miss_falls_back_to_static() {
        let cache = SpotPricingCache {
            fetched_at: chrono::Utc::now(),
            machine_prices: HashMap::new(),
            gpu_prices: HashMap::new(),
        };

        let config = MachineConfig {
            machine_type: "n1-standard-8".into(),
            gpu_type: None,
            gpu_count: None,
            spot: true,
        };
        let est = estimate_pricing(&config, Some(&cache)).unwrap();
        assert!(est.spot_hourly > 0.0);
        assert_eq!(est.breakdown.len(), 2);
    }

    #[test]
    fn cached_g2_machine_works() {
        let mut machine_prices = HashMap::new();
        machine_prices.insert("g2-standard-8".to_string(), 0.544848);

        let cache = SpotPricingCache {
            fetched_at: chrono::Utc::now(),
            machine_prices,
            gpu_prices: HashMap::new(),
        };

        let config = MachineConfig {
            machine_type: "g2-standard-8".into(),
            gpu_type: None,
            gpu_count: None,
            spot: true,
        };
        let est = estimate_pricing(&config, Some(&cache)).unwrap();
        assert_eq!(est.breakdown[0].spot_cost, round2(0.544848));
    }
}
