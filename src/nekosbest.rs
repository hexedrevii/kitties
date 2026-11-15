use kitties::Client;

use crate::models::nekosbest::NekosBestResult;

#[derive(Clone)]
pub struct NekosBest {
  url: String,
  http: reqwest::Client,
}

impl NekosBest {
  pub fn new(url: &str) -> Self {
    Self {
      url: String::from(url),
      http: reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:159.0) Gecko/20100101 Firefox/159.0")
        .build()
        .expect("failed to build HTTP client"),
    }
  }

  pub async fn get(&self, image_type: &str) -> Result<NekosBestResult, reqwest::Error> {
    Ok(
      self
        .get_generic_endpoint::<NekosBestResult>(image_type)
        .await?,
    )
  }
}

impl kitties::Client for NekosBest {
  fn base_url(&self) -> &str {
    &self.url
  }
  fn http(&self) -> &reqwest::Client {
    &self.http
  }
}
