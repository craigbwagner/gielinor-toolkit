use serde::Deserialize;

use crate::client::Rs3Client;
use crate::error::Rs3ApiError;
use crate::models::ItemDetail;

const GE_URL: &str = "https://secure.runescape.com/m=itemdb_rs/api";

// ─────────────────────────────────────────────
// Raw deserialization structs
//
// The GE API wraps everything in an "item" object
// and uses nested objects for price/trend data.
// We flatten this into our clean ItemDetail model.
// ─────────────────────────────────────────────

#[derive(Deserialize)]
struct RawDetailResponse {
    item: RawItem,
}

#[derive(Deserialize)]
struct RawItem {
    id: u32,
    name: String,
    description: String,
    icon: String,
    icon_large: String,
    #[serde(rename = "type")]
    item_type: String,
    current: RawPriceTrend,
    #[allow(dead_code)]
    members: String,
}

#[derive(Deserialize)]
struct RawPriceTrend {
    price: serde_json::Value, // Can be string ("6.7m") or number (0)
}

impl Rs3Client {
    /// Fetches item details from the GE Database.
    /// Returns metadata, icons, and GE guide price (as a display string).
    pub async fn get_item_detail(&self, item_id: u32) -> Result<ItemDetail, Rs3ApiError> {
        let url = format!("{}/catalogue/detail.json?item={}", GE_URL, item_id);
        let response = self.http.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(Rs3ApiError::Parse(format!("Item not found: {}", item_id)));
        }

        let body = response.text().await?;
        let raw: RawDetailResponse = serde_json::from_str(&body).map_err(|e| {
            Rs3ApiError::Parse(format!("Failed to parse GE detail response: {}", e))
        })?;

        Ok(raw.into())
    }
}

impl From<RawDetailResponse> for ItemDetail {
    fn from(raw: RawDetailResponse) -> Self {
        let ge_price = match &raw.item.current.price {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            other => other.to_string(),
        };

        Self {
            id: raw.item.id,
            name: raw.item.name,
            description: raw.item.description,
            icon_url: raw.item.icon,
            icon_large_url: raw.item.icon_large,
            item_type: raw.item.item_type,
            ge_price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_item_detail() {
        let json = r#"{
            "item": {
                "icon": "https://secure.runescape.com/m=itemdb_rs/obj_sprite.gif?id=21787",
                "icon_large": "https://secure.runescape.com/m=itemdb_rs/obj_big.gif?id=21787",
                "id": 21787,
                "type": "Miscellaneous",
                "typeIcon": "https://www.runescape.com/img/categories/Miscellaneous",
                "name": "Steadfast boots",
                "description": "A pair of powerful-looking boots.",
                "current": { "trend": "neutral", "price": "6.7m" },
                "today": { "trend": "neutral", "price": 0 },
                "members": "true",
                "day30": { "trend": "positive", "change": "+9.0%" },
                "day90": { "trend": "positive", "change": "+18.0%" },
                "day180": { "trend": "positive", "change": "+28.0%" }
            }
        }"#;

        let raw: RawDetailResponse = serde_json::from_str(json).unwrap();
        let detail: ItemDetail = raw.into();

        assert_eq!(detail.id, 21787);
        assert_eq!(detail.name, "Steadfast boots");
        assert_eq!(detail.description, "A pair of powerful-looking boots.");
        assert_eq!(detail.item_type, "Miscellaneous");
        assert_eq!(detail.ge_price, "6.7m");
        assert!(detail.icon_url.contains("obj_sprite"));
        assert!(detail.icon_large_url.contains("obj_big"));
    }

    #[test]
    fn handles_numeric_price() {
        let json = r#"{
            "item": {
                "icon": "",
                "icon_large": "",
                "id": 1,
                "type": "Misc",
                "typeIcon": "",
                "name": "Test",
                "description": "Test item",
                "current": { "trend": "neutral", "price": 12345 },
                "today": { "trend": "neutral", "price": 0 },
                "members": "false",
                "day30": { "trend": "neutral", "change": "0%" },
                "day90": { "trend": "neutral", "change": "0%" },
                "day180": { "trend": "neutral", "change": "0%" }
            }
        }"#;

        let raw: RawDetailResponse = serde_json::from_str(json).unwrap();
        let detail: ItemDetail = raw.into();
        assert_eq!(detail.ge_price, "12345");
    }
}
