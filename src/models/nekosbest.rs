use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Dimensions {
  pub width: i32,
  pub height: i32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct NekosBestResult {
  pub results: Vec<NekosBestData>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct NekosBestData {
  pub artist_name: String,
  pub artist_href: String,
  pub source_url: String,
  pub url: String,
  pub dimensions: Dimensions,
}
