use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct PriceResponse {
    pub id: u32,
    pub price: i64,
    pub volume: Option<i64>,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct ItemDetailResponse {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub icon_url: String,
    pub icon_large_url: String,
    pub item_type: String,
    pub ge_price: String,
}

/// Fetches the current trade price for an item from Weird Gloop.
#[tauri::command]
pub async fn get_item_price(
    state: State<'_, AppState>,
    item_id: u32,
) -> Result<PriceResponse, String> {
    let price = state
        .rs3
        .get_item_price(item_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(PriceResponse {
        id: price.id,
        price: price.price,
        volume: price.volume,
        timestamp: price.timestamp,
    })
}

/// Fetches item details from the GE Database (name, icon, description).
#[tauri::command]
pub async fn get_item_detail(
    state: State<'_, AppState>,
    item_id: u32,
) -> Result<ItemDetailResponse, String> {
    let detail = state
        .rs3
        .get_item_detail(item_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ItemDetailResponse {
        id: detail.id,
        name: detail.name,
        description: detail.description,
        icon_url: detail.icon_url,
        icon_large_url: detail.icon_large_url,
        item_type: detail.item_type,
        ge_price: detail.ge_price,
    })
}
