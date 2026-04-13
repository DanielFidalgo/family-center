use anyhow::Result;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;

use crate::configuration::config::IAppConfig;

async fn build_client(config: &dyn IAppConfig) -> Client {
    let creds = Credentials::new(
        config.s3_access_key(),
        config.s3_secret_key(),
        None,
        None,
        "leapcell",
    );

    let sdk_config = aws_config::from_env()
        .region(Region::new(config.s3_region().to_string()))
        .credentials_provider(creds)
        .endpoint_url(config.s3_endpoint())
        .load()
        .await;

    Client::new(&sdk_config)
}

pub async fn upload_avatar(
    config: &dyn IAppConfig,
    person_id: &str,
    content_type: &str,
    data: &[u8],
) -> Result<String> {
    let client = build_client(config).await;
    let key = format!("avatars/{person_id}.jpg");

    client
        .put_object()
        .bucket(config.s3_bucket())
        .key(&key)
        .content_type(content_type)
        .body(ByteStream::from(data.to_vec()))
        .send()
        .await?;

    // Return the public URL
    let url = format!("{}/{}/{}", config.s3_endpoint(), config.s3_bucket(), key);
    Ok(url)
}
