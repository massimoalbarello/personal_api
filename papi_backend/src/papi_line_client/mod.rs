use reqwest::Client;
use std::env;

pub struct PapiLineClient {
    client: Client,
}

impl PapiLineClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn post_download_urls(&self, id: &str, resource: &str, download_url: &str) {
        let body = serde_json::json!({
            "id": id,
            "resource": resource,
            "url": download_url
        });

        let _response = self
            .client
            .post(
                env::var("PAPI_LINE_SERVER_ENDPOINT")
                    .expect("PAPI_LINE_SERVER_ENDPOINT must be set"),
            )
            .json(&body)
            .send()
            .await;
    }
}
