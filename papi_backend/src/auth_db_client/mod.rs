use crate::api::types::OAuthInfo;
use aws_sdk_dynamodb::{
    error::SdkError,
    operation::create_table::CreateTableError,
    types::{
        AttributeDefinition, AttributeValue, BillingMode, KeySchemaElement, KeyType,
        ScalarAttributeType,
    },
    Client,
};
use serde_dynamo::{from_item, to_item};
use std::env;

pub struct AuthDbClient {
    client: Client,
    table_name: String,
}

async fn create_table(
    client: &Client,
    table_name: &str,
    key: &str,
) -> Result<(), SdkError<CreateTableError>> {
    // partition key
    let attr_part = AttributeDefinition::builder()
        .attribute_name(key)
        .attribute_type(ScalarAttributeType::S)
        .build()?;

    let ks = KeySchemaElement::builder()
        .attribute_name(key)
        .key_type(KeyType::Hash)
        .build()?;

    client
        .create_table()
        .table_name(table_name)
        .billing_mode(BillingMode::PayPerRequest)
        .attribute_definitions(attr_part)
        .key_schema(ks)
        .send()
        .await?;

    Ok(())
}

impl AuthDbClient {
    pub async fn setup() -> Result<Self, String> {
        let table_name =
            env::var("DYNAMO_DB_AUTH_TABLE_NAME").expect("DYNAMO_DB_AUTH_TABLE_NAME must be set");

        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        if !client
            .list_tables()
            .send()
            .await
            .map_err(|e| format!("Error listing tables: {}", e.to_string()))?
            .table_names()
            .contains(&table_name)
        {
            create_table(&client, &table_name, "user_id")
                .await
                .map_err(|e| format!("Error creating table: {}", e))?;
            println!("Created table: {}", table_name);
        }

        Ok(Self { client, table_name })
    }

    pub async fn create_auth(&self, oauth_info: OAuthInfo) -> Result<(), String> {
        let item =
            to_item(&oauth_info).map_err(|e| format!("Failed to serialize OAuthInfo: {}", e))?;

        self.client
            .put_item()
            .table_name(&self.table_name)
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
            .table_name(&self.table_name)
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
            .table_name(&self.table_name)
            .item("user_id", AttributeValue::S(user_id))
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| format!("Error updating OAuth info: {}", e))?;

        Ok(())
    }
}
