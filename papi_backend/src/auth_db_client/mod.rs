use crate::api::types::OAuthInfo;
use aws_sdk_dynamodb::{types::AttributeValue, Client};
use serde_dynamo::{from_item, to_item};
use std::env;

pub struct AuthDbClient {
    client: Client,
}

impl AuthDbClient {
    pub async fn setup() -> Result<Self, String> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        Ok(Self { client })
    }

    pub async fn create_auth(&self, oauth_info: OAuthInfo) -> Result<(), String> {
        let item =
            to_item(&oauth_info).map_err(|e| format!("Failed to serialize OAuthInfo: {}", e))?;

        self.client
            .put_item()
            .table_name(
                env::var("DYNAMO_DB_AUTH_TABLE_NAME")
                    .expect("DYNAMO_DB_AUTH_TABLE_NAME must be set"),
            )
            .item("user_id", AttributeValue::S(oauth_info.user_id()))
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| format!("Error inserting oauth info: {}", e))?;

        Ok(())
    }

    pub async fn read_last_auth_for_user(&self, user_id: String) -> Result<OAuthInfo, String> {
        // TODO: make sure that there isn't a better way to query the DB
        let query_output = self
            .client
            .query()
            .table_name(
                env::var("DYNAMO_DB_AUTH_TABLE_NAME")
                    .expect("DYNAMO_DB_AUTH_TABLE_NAME must be set"),
            )
            .key_condition_expression("user_id = :user_id")
            .expression_attribute_values(":user_id", AttributeValue::S(user_id.clone()))
            .scan_index_forward(false)
            .limit(1)
            .send()
            .await
            .map_err(|e| format!("Error querying DB: {}", e))?;

        let item = query_output
            .items
            .unwrap_or_default()
            .into_iter()
            .next()
            .ok_or("No OAuth info found".to_string())?;

        Ok(from_item(item).map_err(|e| format!("Failed to deserialize OAuthInfo: {}", e))?)
    }

    pub async fn update_auth_for_user(
        &self,
        user_id: String,
        oauth_info: OAuthInfo,
    ) -> Result<(), String> {
        let item =
            to_item(&oauth_info).map_err(|e| format!("Failed to serialize OAuthInfo: {}", e))?;

        self.client
            .put_item()
            .table_name(
                env::var("DYNAMO_DB_AUTH_TABLE_NAME")
                    .expect("DYNAMO_DB_AUTH_TABLE_NAME must be set"),
            )
            .item("user_id", AttributeValue::S(user_id))
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| format!("Error updating OAuth info: {}", e))?;

        Ok(())
    }
}
