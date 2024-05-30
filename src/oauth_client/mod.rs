use std::{collections::HashMap, sync::RwLock};

use actix_web::web::Data;
use futures::future::join_all;
use reqwest::Client;
use tokio::time::{interval, Duration};
use types::{
    AccessTokenParams, AccessTokenResponsePayload, AccessTokenUrl, GetArchiveStateParams,
    GetArchiveStateResponsePayload, GetArchiveStateUrl, InitiateArchiveParams,
    InitiateArchiveResponsePayload, InitiateArchiveUrl,
};

use crate::{authorization::types::AuthorizationState, REDIRECT_URI, RESOURCES};

mod types;

pub struct OAuthClient {
    client: Client,
    authorizations: Data<RwLock<HashMap<String, AuthorizationState>>>,
}

impl OAuthClient {
    pub fn new(authorizations: Data<RwLock<HashMap<String, AuthorizationState>>>) -> Self {
        Self {
            client: Client::new(),
            authorizations,
        }
    }
    pub async fn convert_authorization_to_access_token(
        &self,
        client_id: String,
    ) -> Result<(), String> {
        let auth_code = self
            .authorizations
            .read()
            .unwrap()
            .get(&client_id)
            .unwrap()
            .code()
            .unwrap();

        println!("Authrization for client ID {}: {:?}", client_id, auth_code);

        let state = self
            .authorizations
            .read()
            .unwrap()
            .get(&client_id)
            .unwrap()
            .state()
            .to_string();

        let params = AccessTokenParams::default()
            .with_code(auth_code)
            .with_redirect_uri(REDIRECT_URI.to_string())
            .with_state(state);
        let access_token_url = AccessTokenUrl::new(params).as_url();

        println!(
            "Converting auth code to access token for client ID: {}",
            client_id
        );

        let response = self
            .client
            .post(access_token_url)
            .header("Content-Length", 0) // otherwise the server returns 411
            .send()
            .await
            .unwrap();

        if !response.status().is_success() {
            return Err(response.text().await.unwrap());
        }
        let response: AccessTokenResponsePayload = response.json().await.unwrap();

        let access_token = response.access_token();
        println!("Access token: {:?}", access_token);

        self.authorizations
            .write()
            .unwrap()
            .get_mut(&client_id)
            .unwrap()
            .set_access_token(access_token);

        Ok(())
    }

    pub async fn get_data_archive_urls(&self, client_id: String) {
        let initiated_data_archives = RESOURCES
            .iter()
            .map(|&resource| self.initiate_data_archive(client_id.clone(), resource.to_string()));

        let job_ids = join_all(initiated_data_archives).await;

        println!("Job IDs: {:?}", job_ids);

        let get_archive_states = job_ids.iter().filter_map(|job_id| {
            if let Ok((_resource, job_id)) = job_id {
                return Some(self.get_archive_state(client_id.clone(), job_id.clone()));
            }
            None
        });

        let download_urls = join_all(get_archive_states).await;

        println!("Download URLs: {:?}", download_urls);
    }

    async fn initiate_data_archive(
        &self,
        client_id: String,
        resource: String,
    ) -> Result<(String, String), String> {
        println!("Initiating data transfer for resource: {}", resource);

        let params = InitiateArchiveParams::default().with_resources(resource.to_string());
        let initiate_archive_url = InitiateArchiveUrl::new(params).as_url();

        let access_token = self
            .authorizations
            .read()
            .unwrap()
            .get(&client_id)
            .unwrap()
            .access_token()
            .unwrap();

        let response = self
            .client
            .post(initiate_archive_url)
            .bearer_auth(access_token)
            .header("Content-Length", 0) // otherwise the server returns 411
            .send()
            .await
            .unwrap();

        let response: InitiateArchiveResponsePayload = response.json().await.unwrap();

        let job_id = response.archive_job_id();

        println!("Initiated data transfer with job ID: {}", job_id);

        Ok((resource, job_id))
    }

    async fn get_archive_state(&self, client_id: String, job_id: String) -> Result<String, String> {
        let params = GetArchiveStateParams::default();
        let get_archive_state_url = GetArchiveStateUrl::new(job_id.clone(), params).as_url();

        let mut interval = interval(Duration::from_secs(10));

        loop {
            println!("Checking state for job ID: {}", job_id);
            interval.tick().await;

            let access_token = self
                .authorizations
                .read()
                .unwrap()
                .get(&client_id)
                .unwrap()
                .access_token()
                .unwrap();

            let response = self
                .client
                .get(get_archive_state_url.clone())
                .bearer_auth(access_token)
                .header("Content-Length", 0) // otherwise the server returns 411
                .send()
                .await
                .unwrap();

            println!("Response: {:?}", response);

            match response.json::<GetArchiveStateResponsePayload>().await {
                Ok(response) => {
                    println!("Job state: {:?}", response.state());
                    let download_url = response.urls()[0].clone();
                    println!("Download URL for job ID {}: {:?}", job_id, download_url);
                    return Ok(download_url);
                }
                Err(_) => {
                    // TODO: distinguish the case in which the server returns an error
                    //       from the ones in which the job has not yet finished or has failed
                    //       for now we just assume that each job eventually completes
                }
            }
        }
    }
}
