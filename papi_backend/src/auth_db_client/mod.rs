use futures::TryStreamExt;
use mongodb::{bson::doc, Client};

use crate::api::types::OAuthInfo;

const AUTH_DB_NAME: &str = "papi_auth_db";
const AUTH_COLL_NAME: &str = "user_authorizations";

pub struct AuthDbClient {
    client: Client,
}

impl AuthDbClient {
    pub async fn setup() -> Result<Self, String> {
        let auth_db_uri = std::env::var("AUTHORIZATION_DB_URI")
            .map_err(|_| "AUTHORIZATION_DB_URI must be set")?;

        let client = Client::with_uri_str(auth_db_uri)
            .await
            .map_err(|e| format!("failed to connect to DB: {:?}", e.to_string()))?;

        client
            .database(AUTH_DB_NAME)
            .create_collection(AUTH_COLL_NAME)
            .await
            .map_err(|e| format!("failed to create collection: {:?}", e))?;

        println!("Created collection {}", AUTH_COLL_NAME);

        Ok(Self { client })
    }

    pub async fn create_auth(&self, oauth_info: OAuthInfo) -> Result<(), String> {
        self.client
            .database(AUTH_DB_NAME)
            .collection(AUTH_COLL_NAME)
            .insert_one(oauth_info)
            .await
            .map_err(|e| format!("Error inserting oauth info: {}", e))?;
        Ok(())
    }

    pub async fn read_last_auth_for_user(&self, user_id: String) -> Result<OAuthInfo, String> {
        let mut cursor = self
            .client
            .database(AUTH_DB_NAME)
            .collection::<OAuthInfo>(AUTH_COLL_NAME)
            .find(doc! {"user_id": user_id.clone()})
            .sort(doc! { "created_at": -1 })
            .limit(1)
            .await
            .map_err(|e| format!("Error querying DB: {}", e))?;

        cursor
            .try_next()
            .await
            .map_err(|e| format!("Error resolving next item in stream: {}", e))?
            .ok_or("No OAuth info found".to_string())
    }

    pub async fn update_auth_for_user(
        &self,
        user_id: String,
        oauth_info: OAuthInfo,
    ) -> Result<(), String> {
        self.client
            .database(AUTH_DB_NAME)
            .collection(AUTH_COLL_NAME)
            .replace_one(doc! {"user_id": user_id.clone()}, oauth_info)
            .await
            .map_err(|e| format!("Error updating OAuth info: {}", e))?;
        Ok(())
    }
}
