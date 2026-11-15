use serde::de::DeserializeOwned;

pub trait Client: Send + Sync {
  fn base_url(&self) -> &str;
  fn http(&self) -> &reqwest::Client;

  fn get_generic_endpoint<T>(
    &self,
    endpoint: &str,
  ) -> impl Future<Output = Result<T, reqwest::Error>> + Send
  where
    T: DeserializeOwned,
  {
    async move {
      let mut url = String::from(self.base_url());
      if !url.ends_with('/') {
        url.push('/');
      }
      url.push_str(endpoint.trim_start_matches('/'));

      self
        .http()
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json::<T>()
        .await
    }
  }
}
