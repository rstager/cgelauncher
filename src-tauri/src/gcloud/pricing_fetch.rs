use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const SPOT_PRICING_URL: &str = "https://cloud.google.com/spot-vms/pricing";
const CACHE_TTL_HOURS: i64 = 24;

/// Cached spot pricing data fetched from Google's pricing page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotPricingCache {
    pub fetched_at: DateTime<Utc>,
    /// All-in spot price per hour keyed by machine type (e.g. "n1-standard-8" -> 0.10446).
    pub machine_prices: HashMap<String, f64>,
    /// Per-GPU spot price per hour keyed by GPU short name (e.g. "T4" -> 0.175).
    pub gpu_prices: HashMap<String, f64>,
}

impl SpotPricingCache {
    pub fn is_expired(&self) -> bool {
        let age = Utc::now() - self.fetched_at;
        age.num_hours() >= CACHE_TTL_HOURS
    }
}

/// Fetch and parse spot pricing from Google's pricing page.
pub async fn fetch_spot_pricing() -> Result<SpotPricingCache, String> {
    let html = reqwest::get(SPOT_PRICING_URL)
        .await
        .map_err(|e| format!("Failed to fetch pricing page: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Failed to read pricing response: {e}"))?;

    parse_spot_pricing_html(&html)
}

/// Parse the HTML from the spot pricing page into structured data.
pub fn parse_spot_pricing_html(html: &str) -> Result<SpotPricingCache, String> {
    let document = Html::parse_document(html);
    let table_sel = Selector::parse("table").map_err(|e| format!("Bad selector: {e}"))?;
    let tr_sel = Selector::parse("tr").map_err(|e| format!("Bad selector: {e}"))?;
    let td_sel = Selector::parse("td").map_err(|e| format!("Bad selector: {e}"))?;
    let th_sel = Selector::parse("th").map_err(|e| format!("Bad selector: {e}"))?;

    let mut machine_prices: HashMap<String, f64> = HashMap::new();
    let mut gpu_prices: HashMap<String, f64> = HashMap::new();

    for table in document.select(&table_sel) {
        let headers: Vec<String> = table
            .select(&th_sel)
            .map(|th| th.text().collect::<String>().trim().to_string())
            .collect();

        if headers.is_empty() {
            continue;
        }

        let is_gpu_table = headers.len() == 2
            && headers[0].contains("GPU")
            && headers[1].contains("Spot");

        let is_machine_table = headers.len() == 4
            && headers[0].contains("Machine type")
            && headers.iter().any(|h| h.contains("Spot"));

        let is_accelerator_machine_table = headers.len() == 4
            && headers[0].contains("Machine type")
            && headers.iter().any(|h| h.contains("GPU"))
            && headers.iter().any(|h| h.contains("Spot"));

        if is_gpu_table {
            // Per-GPU accelerator pricing (2 columns: GPU name, spot price)
            for row in table.select(&tr_sel) {
                let cells: Vec<String> = row
                    .select(&td_sel)
                    .map(|td| td.text().collect::<String>().trim().to_string())
                    .collect();
                if cells.len() == 2 {
                    let gpu_name = cells[0].trim().to_string();
                    if let Some(price) = parse_price(&cells[1]) {
                        if !gpu_name.is_empty() {
                            gpu_prices.insert(gpu_name, price);
                        }
                    }
                }
            }
        } else if is_accelerator_machine_table {
            // All-in GPU machine types (A2, A3, G2, etc.) — 4 columns with GPU column
            for row in table.select(&tr_sel) {
                let cells: Vec<String> = row
                    .select(&td_sel)
                    .map(|td| td.text().collect::<String>().trim().to_string())
                    .collect();
                if cells.len() == 4 {
                    let machine_type = cells[0].trim().to_string();
                    if let Some(price) = parse_price(&cells[3]) {
                        if looks_like_machine_type(&machine_type) {
                            machine_prices.insert(machine_type, price);
                        }
                    }
                }
            }
        } else if is_machine_table {
            // Standard machine type table (4 columns: type, vcpus, memory, price)
            for row in table.select(&tr_sel) {
                let cells: Vec<String> = row
                    .select(&td_sel)
                    .map(|td| td.text().collect::<String>().trim().to_string())
                    .collect();
                if cells.len() == 4 {
                    let machine_type = cells[0].trim().to_string();
                    if let Some(price) = parse_price(&cells[3]) {
                        if looks_like_machine_type(&machine_type) {
                            machine_prices.insert(machine_type, price);
                        }
                    }
                }
            }
        }
    }

    if machine_prices.is_empty() && gpu_prices.is_empty() {
        return Err("No pricing data found on page — format may have changed".into());
    }

    Ok(SpotPricingCache {
        fetched_at: Utc::now(),
        machine_prices,
        gpu_prices,
    })
}

/// Extract a dollar price from strings like "$0.10446 / 1 hour".
fn parse_price(s: &str) -> Option<f64> {
    let s = s.trim();
    let s = s.strip_prefix('$')?;
    let price_part = s.split('/').next()?.trim();
    // Remove commas from numbers like "$1,360"
    let clean = price_part.replace(',', "");
    clean.parse::<f64>().ok()
}

/// Heuristic: machine types contain a hyphen and start with a letter.
fn looks_like_machine_type(s: &str) -> bool {
    let s = s.trim();
    s.contains('-')
        && s.chars().next().map_or(false, |c| c.is_ascii_lowercase())
        && !s.contains(' ')
}

/// Map GPU short names from the pricing page to the gcloud accelerator type names.
pub fn gpu_page_name_to_gcloud(page_name: &str) -> Option<&'static str> {
    match page_name {
        "T4" => Some("nvidia-tesla-t4"),
        "V100" => Some("nvidia-tesla-v100"),
        "P100" => Some("nvidia-tesla-p100"),
        "P4" => Some("nvidia-tesla-p4"),
        "A100" => Some("nvidia-tesla-a100"),
        "L4" => Some("nvidia-l4"),
        _ => None,
    }
}

/// Map gcloud accelerator type names to the page's short name for lookup.
pub fn gcloud_gpu_to_page_name(gcloud_name: &str) -> Option<&'static str> {
    match gcloud_name {
        "nvidia-tesla-t4" => Some("T4"),
        "nvidia-tesla-v100" => Some("V100"),
        "nvidia-tesla-p100" => Some("P100"),
        "nvidia-tesla-p4" => Some("P4"),
        "nvidia-tesla-a100" => Some("A100"),
        "nvidia-l4" => Some("L4"),
        _ => None,
    }
}

fn cache_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("cgelauncher").join("spot_pricing_cache.json"))
}

/// Load cached pricing from disk, returning None if missing or expired.
pub fn load_cache() -> Option<SpotPricingCache> {
    let path = cache_path()?;
    let data = std::fs::read_to_string(&path).ok()?;
    let cache: SpotPricingCache = serde_json::from_str(&data).ok()?;
    if cache.is_expired() {
        return None;
    }
    Some(cache)
}

/// Save pricing cache to disk.
pub fn save_cache(cache: &SpotPricingCache) -> Result<(), String> {
    let path = cache_path().ok_or("Cannot determine app data directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create cache directory: {e}"))?;
    }
    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| format!("Failed to serialize cache: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write cache: {e}"))?;
    Ok(())
}

/// Load from cache if valid, otherwise fetch from web and cache the result.
pub async fn get_spot_pricing() -> Option<SpotPricingCache> {
    if let Some(cached) = load_cache() {
        return Some(cached);
    }
    match fetch_spot_pricing().await {
        Ok(cache) => {
            if let Err(e) = save_cache(&cache) {
                eprintln!("Warning: failed to save pricing cache: {e}");
            }
            Some(cache)
        }
        Err(e) => {
            eprintln!("Warning: failed to fetch spot pricing: {e}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_price_standard() {
        assert_eq!(parse_price("$0.10446 / 1 hour"), Some(0.10446));
    }

    #[test]
    fn parse_price_with_comma() {
        assert_eq!(parse_price("$1,360.50 / 1 hour"), Some(1360.50));
    }

    #[test]
    fn parse_price_simple() {
        assert_eq!(parse_price("$0.175 / 1 hour"), Some(0.175));
    }

    #[test]
    fn parse_price_no_dollar() {
        assert_eq!(parse_price("0.175 / 1 hour"), None);
    }

    #[test]
    fn looks_like_machine_type_valid() {
        assert!(looks_like_machine_type("n1-standard-8"));
        assert!(looks_like_machine_type("a2-highgpu-1g"));
        assert!(looks_like_machine_type("g2-standard-8"));
    }

    #[test]
    fn looks_like_machine_type_invalid() {
        assert!(!looks_like_machine_type("A2 Standard"));
        assert!(!looks_like_machine_type(""));
        assert!(!looks_like_machine_type("GPU"));
    }

    #[test]
    fn gpu_name_mapping_roundtrip() {
        for page_name in &["T4", "V100", "P100", "P4", "L4"] {
            let gcloud = gpu_page_name_to_gcloud(page_name).unwrap();
            let back = gcloud_gpu_to_page_name(gcloud).unwrap();
            assert_eq!(back, *page_name);
        }
    }

    #[test]
    fn parse_real_html_tables() {
        let html = r#"
        <html><body>
        <table class="nooFgd">
            <tr><th>Machine type</th><th>Virtual CPUs</th><th>Memory</th><th>Current Spot pricing (USD)</th></tr>
            <tbody>
            <tr><td><p>n1-standard-1</p></td><td><p>1</p></td><td><p>3.75 GiB</p></td><td>$0.0130575 / 1 hour</td></tr>
            <tr><td><p>n1-standard-8</p></td><td><p>8</p></td><td><p>30 GiB</p></td><td>$0.10446 / 1 hour</td></tr>
            </tbody>
        </table>
        <table class="nooFgd">
            <tr><th><p><b>GPU</b></p></th><th>Current Spot GPU pricing (USD)</th></tr>
            <tbody>
            <tr><td><p>V100</p></td><td>$1.0885 / 1 hour</td></tr>
            <tr><td><p>T4</p></td><td>$0.175 / 1 hour</td></tr>
            </tbody>
        </table>
        <table class="nooFgd">
            <tr><th><p><b>Machine type</b></p></th><th><p><b>GPU</b></p></th><th>Components</th><th>Current Spot pricing (USD)</th></tr>
            <tbody>
            <tr><td><p>a2-highgpu-1g</p></td><td><p>Nvidia A100</p></td><td><p>GPUs: 1</p></td><td>$1.80385 / 1 hour</td></tr>
            </tbody>
        </table>
        </body></html>
        "#;

        let cache = parse_spot_pricing_html(html).unwrap();

        assert_eq!(cache.machine_prices.get("n1-standard-1"), Some(&0.0130575));
        assert_eq!(cache.machine_prices.get("n1-standard-8"), Some(&0.10446));
        assert_eq!(cache.machine_prices.get("a2-highgpu-1g"), Some(&1.80385));
        assert_eq!(cache.gpu_prices.get("V100"), Some(&1.0885));
        assert_eq!(cache.gpu_prices.get("T4"), Some(&0.175));
    }

    #[test]
    fn parse_empty_html_returns_error() {
        let result = parse_spot_pricing_html("<html><body></body></html>");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No pricing data"));
    }

    #[test]
    fn cache_expiry() {
        let fresh = SpotPricingCache {
            fetched_at: Utc::now(),
            machine_prices: HashMap::new(),
            gpu_prices: HashMap::new(),
        };
        assert!(!fresh.is_expired());

        let old = SpotPricingCache {
            fetched_at: Utc::now() - chrono::Duration::hours(25),
            machine_prices: HashMap::new(),
            gpu_prices: HashMap::new(),
        };
        assert!(old.is_expired());
    }
}
