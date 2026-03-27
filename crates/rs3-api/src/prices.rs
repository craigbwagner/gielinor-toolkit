use std::collections::HashMap;

use serde::Deserialize;

use crate::client::Rs3Client;
use crate::error::Rs3ApiError;
use crate::models::ItemPrice;

const PRICES_URL: &str = "https://api.weirdgloop.org/exchange/history/rs";

// ─────────────────────────────────────────────
// Raw deserialization structs
//
// The Weird Gloop API returns item IDs as string keys:
// { "21787": { "id": "21787", "price": 6704921, ... } }
// ─────────────────────────────────────────────

#[derive(Deserialize)]
struct RawPriceEntry {
    id: String,
    price: i64,
    volume: Option<i64>,
    timestamp: String,
}

impl Rs3Client {
    /// Fetches the latest trade price for a single item.
    pub async fn get_item_price(&self, item_id: u32) -> Result<ItemPrice, Rs3ApiError> {
        let prices = self.get_item_prices(&[item_id]).await?;
        prices
            .into_iter()
            .next()
            .ok_or_else(|| Rs3ApiError::Parse(format!("No price data for item {}", item_id)))
    }

    /// Fetches latest trade prices for multiple items (up to 100).
    /// Returns a Vec since some items may not have price data.
    pub async fn get_item_prices(&self, item_ids: &[u32]) -> Result<Vec<ItemPrice>, Rs3ApiError> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }
        if item_ids.len() > 100 {
            return Err(Rs3ApiError::Parse(
                "Weird Gloop API supports max 100 items per request".to_string(),
            ));
        }

        let ids_param = item_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join("|");

        let url = format!("{}/latest?id={}", PRICES_URL, ids_param);
        let response = self.http.get(&url).send().await?;
        let body = response.text().await?;

        // Response is a map of string IDs to price entries
        let raw: HashMap<String, RawPriceEntry> =
            serde_json::from_str(&body).map_err(|e| {
                Rs3ApiError::Parse(format!("Failed to parse Weird Gloop response: {}", e))
            })?;

        let prices = raw
            .into_values()
            .map(|entry| {
                let id = entry.id.parse::<u32>().unwrap_or(0);
                ItemPrice {
                    id,
                    price: entry.price,
                    volume: entry.volume,
                    timestamp: entry.timestamp,
                }
            })
            .collect();

        Ok(prices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_single_item_response() {
        let json = r#"{"21787":{"id":"21787","timestamp":"2026-03-27T15:54:14.000Z","price":6704921,"volume":21}}"#;

        let raw: HashMap<String, RawPriceEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(raw.len(), 1);

        let entry = &raw["21787"];
        assert_eq!(entry.id, "21787");
        assert_eq!(entry.price, 6704921);
        assert_eq!(entry.volume, Some(21));
    }

    #[test]
    fn deserializes_multi_item_response() {
        let json = r#"{
            "21787": {"id":"21787","timestamp":"2026-03-27T15:54:14.000Z","price":6704921,"volume":21},
            "21790": {"id":"21790","timestamp":"2026-03-27T15:54:14.000Z","price":950000,"volume":null}
        }"#;

        let raw: HashMap<String, RawPriceEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(raw.len(), 2);
        assert_eq!(raw["21790"].volume, None);
    }
}
