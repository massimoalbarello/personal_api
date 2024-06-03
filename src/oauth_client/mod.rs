use std::{collections::HashMap, sync::RwLock};

use actix_web::{web::Data, Result};
use reqwest::Client;
use tokio::{
    sync::mpsc::UnboundedSender,
    time::{interval, Duration},
};
use types::{
    AccessTokenParams, AccessTokenResponsePayload, AccessTokenUrl, GetArchiveStateParams,
    GetArchiveStateResponsePayload, GetArchiveStateUrl, InitiateArchiveParams,
    InitiateArchiveResponsePayload, InitiateArchiveUrl, ResetAuthorizationParams,
    ResetAuthorizationResponsePayload, ResetAuthorizationUrl,
};

use crate::{authorization::types::AuthorizationState, REDIRECT_URI, RESOURCES};

mod types;

pub struct OAuthClient {
    client: Client,
    authorizations: Data<RwLock<HashMap<String, AuthorizationState>>>,
    download_info_tx: UnboundedSender<((String, String), Result<String, String>)>,
}

impl OAuthClient {
    pub fn new(
        authorizations: Data<RwLock<HashMap<String, AuthorizationState>>>,
        download_info_tx: UnboundedSender<((String, String), Result<String, String>)>,
    ) -> Self {
        Self {
            client: Client::new(),
            authorizations,
            download_info_tx,
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

    pub fn initiate_data_archives(&self, client_id: String) {
        let access_token = self
            .authorizations
            .read()
            .unwrap()
            .get(&client_id)
            .unwrap()
            .access_token()
            .unwrap();

        for resource in RESOURCES {
            let oauth_client = Client::clone(&self.client);
            let access_token = access_token.clone();
            let download_info_tx = self.download_info_tx.clone();
            let client_id = client_id.clone();
            tokio::spawn(async move {
                match initiate_data_archive(
                    oauth_client.clone(),
                    resource.to_string(),
                    access_token.clone(),
                )
                .await
                {
                    Ok((resource, job_id)) => {
                        match poll_archive_state(oauth_client.clone(), job_id.clone(), access_token)
                            .await
                        {
                            Ok(download_url) => download_info_tx
                                .send(((client_id, resource), Ok(download_url)))
                                .unwrap(),
                            Err(e) => download_info_tx
                                .send(((client_id, resource), Err(e)))
                                .unwrap(),
                        }
                    }
                    Err(e) => download_info_tx
                        .send(((client_id, resource.to_string()), Err(e)))
                        .unwrap(),
                }
            });
        }
    }

    pub async fn reset_authorization(&self, client_id: String) -> Result<(), String> {
        let params = ResetAuthorizationParams::default();
        let reset_authorization_url = ResetAuthorizationUrl::new(params).as_url();

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
            .post(reset_authorization_url)
            .bearer_auth(access_token)
            .header("Content-Length", 0) // otherwise the server returns 411
            .send()
            .await
            .unwrap();

        let _: ResetAuthorizationResponsePayload = response.json().await.unwrap();

        println!("Reset authorization for client ID: {}", client_id);

        Ok(())
    }
}

async fn initiate_data_archive(
    oauth_client: Client,
    resource: String,
    access_token: String,
) -> Result<(String, String), String> {
    println!("Initiating data transfer for resource: {}", resource);

    let params = InitiateArchiveParams::default().with_resources(resource.to_string());
    let initiate_archive_url = InitiateArchiveUrl::new(params).as_url();

    let response = oauth_client
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

async fn poll_archive_state(
    oauth_client: Client,
    job_id: String,
    access_token: String,
) -> Result<String, String> {
    let params = GetArchiveStateParams::default();
    let poll_archive_state_url = GetArchiveStateUrl::new(job_id.clone(), params).as_url();

    let mut interval = interval(Duration::from_secs(10));
    loop {
        println!("Checking state for job ID: {}", job_id);
        interval.tick().await;

        let response = oauth_client
            .get(poll_archive_state_url.clone())
            .bearer_auth(access_token.clone())
            .header("Content-Length", 0) // otherwise the server returns 411
            .send()
            .await
            .unwrap();

        match response.json::<GetArchiveStateResponsePayload>().await {
            Ok(GetArchiveStateResponsePayload::Completed(response)) => {
                let download_url = response.urls()[0].clone();
                println!(
                    "Job with ID {} completed. Download URL: {:?}",
                    job_id, download_url
                );
                return Ok(download_url);
            }
            Ok(GetArchiveStateResponsePayload::InProgress(_)) => {
                println!("Job with ID {} still in progress", job_id);
            }
            Err(e) => {
                // TODO: distinguish the case in which the server returns an error
                //       from the ones in which the job has failed
                //       for now we just assume that each job eventually completes
                let error = format!("Job with ID {} failed: {:?}", job_id, e);
                println!("{}", error);
                return Err(error);
            }
        }
    }
}
