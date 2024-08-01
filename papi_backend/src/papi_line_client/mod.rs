use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, Response};
use std::env;
use std::fs::File;
use std::io::{self, Write};
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

    pub async fn download_file(
        &self,
        id: String,
        resource: &String,
        url: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self.client.get(url).send().await?;

        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("");

        if ZIP_MIME_TYPES.contains(&content_type) {
            println!("Unzipping files");
            let filenames = unzip_and_flatten(&id, response).await?;
            Ok(filenames)
        } else {
            let content_disposition = response
                .headers()
                .get("Content-Disposition")
                .and_then(|v| v.to_str().ok());
            println!("File is not a ZIP file:\n{:?}", response.headers());

            if let Some(content_disposition) = content_disposition {
                if let Some(filename) = extract_filename(content_disposition) {
                    println!("Saving file: {}", filename);
                    let file_path = Path::new(USERS_DATALAKE).join(&filename);
                    let mut file = File::create(file_path)?;
                    let content = response.bytes().await?;
                    file.write_all(&content)?;
                    println!("📁 File {} saved successfully", filename);
                    Ok(vec![filename])
                } else {
                    println!("❗️ The file from the url is not a zip file and does not have a valid filename.");
                    Ok(vec![])
                }
            } else {
                println!("❗️ The file from the url is not a zip file and does not have a valid filename.");
                Ok(vec![])
            }
        }
    }
}

async fn unzip_and_flatten(
    id: &str,
    response: Response,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut zip = ZipArchive::new(io::Cursor::new(response.bytes().await?))?;
    let mut filenames = Vec::new();

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let filename = file.name().to_string();
        let file_path = Path::new(USERS_DATALAKE).join(&filename);

        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut outfile = File::create(&file_path)?;
        io::copy(&mut file, &mut outfile)?;
        filenames.push(filename);
    }

    Ok(filenames)
}

fn extract_filename(content_disposition: &str) -> Option<String> {
    if let Some(start) = content_disposition.find("filename=") {
        let filename = &content_disposition[start + 9..].trim_matches('"');
        Some(filename.to_string())
    } else {
        None
    }
}
