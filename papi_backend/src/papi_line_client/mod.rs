use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::create_bucket::{CreateBucketError, CreateBucketOutput};
use aws_sdk_s3::types::{BucketLocationConstraint, CreateBucketConfiguration};
use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
use chrono::Utc;
use reqwest::{Client as ReqwestClient, Response};
use std::error::Error;
use std::io::Cursor;
use std::{env, io::Read};
use zip::read::ZipArchive;

const ZIP_MIME_TYPES: [&str; 4] = [
    "application/zip",
    "application/x-zip",
    "application/x-zip-compressed",
    "multipart/x-zip",
];

async fn create_bucket(
    client: &S3Client,
    bucket: &str,
    region: &str,
) -> Result<CreateBucketOutput, SdkError<CreateBucketError>> {
    let constraint = BucketLocationConstraint::from(region);
    let cfg = CreateBucketConfiguration::builder()
        .location_constraint(constraint)
        .build();

    client
        .create_bucket()
        .create_bucket_configuration(cfg)
        .bucket(bucket)
        .send()
        .await
}

pub struct PapiLineClient {
    request_client: ReqwestClient,
    s3_client: aws_sdk_s3::Client,
    bucket_name: String,
}

impl PapiLineClient {
    pub async fn setup() -> Result<Self, String> {
        let bucket_name = env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set");

        let config = aws_config::load_from_env().await;
        let s3_client = S3Client::new(&config);

        if !s3_client
            .list_buckets()
            .send()
            .await
            .map_err(|e| format!("Error listing buckets: {}", e.to_string()))?
            .buckets()
            .iter()
            .any(|b| b.name() == Some(&bucket_name))
        {
            create_bucket(
                &s3_client,
                &bucket_name,
                env::var("AWS_REGION")
                    .expect("AWS_REGION must be set")
                    .as_str(),
            )
            .await
            .map_err(|e| format!("Error creating table: {}", e))?;
            println!("Created bucket: {}", bucket_name);
        }

        Ok(Self {
            request_client: ReqwestClient::new(),
            s3_client,
            bucket_name,
        })
    }

    pub async fn download_file(
        &self,
        user_id: String,
        resource: &String,
        url: &str,
    ) -> Result<(), String> {
        let response = self
            .request_client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("could not request data download: {:?}", e.to_string()))?;

        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("");

        if ZIP_MIME_TYPES.contains(&content_type) {
            println!("Unzipping files for resource: {}", resource);
            self.unzip_and_flatten(&user_id, resource, response)
                .await
                .map_err(|e| format!("could not unzip files: {:?}", e.to_string()))
        } else {
            Err(format!("file is not a ZIP file:{:?}", response.headers()))
        }
    }

    async fn unzip_and_flatten(
        &self,
        user_id: &str,
        resource: &String,
        response: Response,
    ) -> Result<(), Box<dyn Error>> {
        let mut zip = ZipArchive::new(Cursor::new(response.bytes().await?))?;

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
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                let body = ByteStream::from(buffer);
                self.s3_client
                    .put_object()
                    .bucket(&self.bucket_name)
                    .key(filename)
                    .body(body)
                    .send()
                    .await?;
            } else {
                println!("Error parsing file path: {:?}", file.name());
            }
        }
        Ok(())
    }
}
