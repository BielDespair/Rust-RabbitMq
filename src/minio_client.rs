
use std::{env, error::Error, sync::OnceLock};

use minio::s3::{creds::{Provider, StaticProvider}, http::BaseUrl, types::S3Api, Client, ClientBuilder};

pub struct MinioVariables {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String
}

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn init_client(variables: &MinioVariables) -> &'static Client {

     
    let base_url: BaseUrl = variables.endpoint.parse::<BaseUrl>().expect("invalid base url");

    let static_provider = StaticProvider::new(
        &variables.access_key,
        &variables.secret_key,
        None,
    );

    CLIENT.get_or_init(|| {
        ClientBuilder::new(base_url)
            .provider(Some(Box::new(static_provider) as Box<dyn Provider + Send + Sync>))
            .build()
            .expect("failed to build minio client")
    })
}

pub async fn download_object(object: &str, variables: &MinioVariables) -> Result<String, Box<dyn Error>> {
    let client: &Client = CLIENT.get().expect("Client not initialized");

    let mut resp: minio::s3::response::GetObjectResponse = client.get_object(&variables.bucket_name, object).send().await?;

    let content_bytes = resp.content.to_segmented_bytes().await?.to_bytes();
    let content_string: String = String::from_utf8(content_bytes.to_vec())?;

    Ok(content_string)
}


pub fn initialize_variables() -> MinioVariables {

    MinioVariables {
        endpoint: env::var("MINIO_ENDPOINT").expect(&env_not_present(&"MINIO_ENDPOINT")),
        access_key: env::var("MINIO_ACCESS_KEY").expect(&env_not_present(&"MINIO_ACCESS_KEY")),
        secret_key: env::var("MINIO_SECRET_KEY").expect(&env_not_present(&"MINIO_SECRET_KEY")),
        bucket_name: env::var("MINIO_BUCKET_NAME").expect(&env_not_present(&"MINIO_BUCKET_NAME")),
    }
}

#[inline]
fn env_not_present(var_name: &str) -> String {
    return format!("MinIO enviroment variable '{}' not set", var_name);
}