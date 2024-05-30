use std::{collections::HashMap, sync::RwLock};

use actix_web::web::Data;
use reqwest::Client;
use types::{
    AccessTokenParams, AccessTokenResponsePayload, AccessTokenUrl, InitiateTransferParams,
    InitiateTransferResponsePayload, InitiateTransferUrl,
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
    pub async fn convert_auth_code_to_access_token(&self, client_id: String) -> Result<(), String> {
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

    pub async fn get_data_transfer_urls(&self, client_id: String) {
        for resource in RESOURCES.iter() {
            println!("Initiating data transfer for resource: {}", resource);

            let params = InitiateTransferParams::default().with_resources(resource.to_string());
            let initiate_transfer_url = InitiateTransferUrl::new(params).as_url();

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
                .post(initiate_transfer_url)
                .bearer_auth(access_token)
                .header("Content-Length", 0) // otherwise the server returns 411
                .send()
                .await
                .unwrap();

            let response: InitiateTransferResponsePayload = response.json().await.unwrap();

            let job_id = response.archive_job_id();

            println!("Initiated data transfer with job ID: {}", job_id);
        }
    }
}
