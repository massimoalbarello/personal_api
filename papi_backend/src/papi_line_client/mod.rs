use chrono::Utc;
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Response};
use std::env;
use std::fs::File;
use std::io::{self, Cursor};
use std::path::Path;
use zip::read::ZipArchive;

const USERS_DATALAKE: &str = "./users_datalake";
const ZIP_MIME_TYPES: [&str; 4] = [
    "application/zip",
    "application/x-zip",
    "application/x-zip-compressed",
    "multipart/x-zip",
];

pub struct PapiLineClient {
    client: Client,
}

impl PapiLineClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn post_download_urls(&self, user_id: &str, resource: &str, download_url: &str) {
        let body = serde_json::json!({
            "id": user_id,
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

    pub async fn download_file(
        &self,
        user_id: String,
        resource: &String,
        url: &str,
    ) -> Result<Vec<String>, String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("could not request data download: {:?}", e.to_string()))?;

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("");

        if ZIP_MIME_TYPES.contains(&content_type) {
            println!("Unzipping files for resource: {}", resource);
            let filenames = unzip_and_flatten(&user_id, resource, response)
                .await
                .map_err(|e| format!("could not unzip files: {:?}", e.to_string()))?;
            Ok(filenames)
        } else {
            Err(format!("file is not a ZIP file:{:?}", response.headers()))
        }
    }
}

async fn unzip_and_flatten(
    user_id: &str,
    resource: &String,
    response: Response,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut zip = ZipArchive::new(Cursor::new(response.bytes().await?))?;
    let mut filenames = Vec::new();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;

        if let Some(filename) = file.name().split('/').last() {
            println!("Extracting file: {:?}", filename);
            let filename = format!(
                "{}_{}_{}_{}",
                Utc::now().timestamp(),
                user_id,
                resource,
                filename
            );
            let file_path = Path::new(USERS_DATALAKE).join(&filename);

            let mut outfile = File::create(&file_path)?;
            io::copy(&mut file, &mut outfile)?;
            filenames.push(filename);
        } else {
            println!("Error parsing file path: {:?}", file.name());
        }
    }

    Ok(filenames)
}
