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

    pub async fn post_download_urls(&self, download_urls: Vec<Result<String, String>>) {
        let _response = self
            .client
            .post(PAPI_LINE_SERVER_ENDPOINT)
            .json(&download_urls)
            .send()
            .await;
    }
}
