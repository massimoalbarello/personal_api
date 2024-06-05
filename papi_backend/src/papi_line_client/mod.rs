use reqwest::Client;

const PAPI_LINE_SERVER_ENDPOINT: &str = "http://localhost:6969/download";

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
            .post(PAPI_LINE_SERVER_ENDPOINT)
            .json(&body)
            .send()
            .await;
    }
}
