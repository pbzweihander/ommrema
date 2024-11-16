use once_cell::sync::Lazy;
use serde::Deserialize;
use url::Url;

pub static CONFIG: Lazy<Config> =
    Lazy::new(|| envy::from_env().expect("failed to parse config from environment variables"));
pub static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .user_agent(env!("CARGO_PKG_NAME"))
        .build()
        .unwrap()
});
pub static OBJECT_STORE: Lazy<object_store::aws::AmazonS3> = Lazy::new(|| {
    object_store::aws::AmazonS3Builder::from_env()
        .with_bucket_name(CONFIG.s3_bucket_name.clone())
        .build()
        .expect("failed to build S3 object store")
});

fn deserialize_jwt_secret<'de, D>(
    d: D,
) -> Result<(jsonwebtoken::EncodingKey, jsonwebtoken::DecodingKey), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    Ok((
        jsonwebtoken::EncodingKey::from_secret(s.as_bytes()),
        jsonwebtoken::DecodingKey::from_secret(s.as_bytes()),
    ))
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub title: String,

    pub public_url: Url,

    #[serde(deserialize_with = "deserialize_jwt_secret")]
    pub jwt_secret: (jsonwebtoken::EncodingKey, jsonwebtoken::DecodingKey),

    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub discord_guild_id: String,
    pub discord_guild_role_id: String,

    pub s3_bucket_name: String,
}
