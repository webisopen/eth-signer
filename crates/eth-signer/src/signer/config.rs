use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SignerConfig {
    PrivateKey(String),
    Mnemonic(String),
    KeyStore {
        path: String,
        password: String,
    },
    AzureKeyVault {
        key: String,
        secret: String,
    },
    AwsKms {
        key: String,
    },
    GoogleKms {
        project_id: String,
        location: String,
        key_ring: String,
        key: String,
        version: u64,
    },
    AlicloudKms {
        key: String,
        secret: String,
    },
}

impl fmt::Debug for SignerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SignerConfig::PrivateKey(_) => f.write_str("PrivateKey(...)"),
            SignerConfig::Mnemonic(_) => f.write_str("Mnemonic(...)"),
            SignerConfig::KeyStore { path, password: _ } => f
                .debug_struct("KeyStore")
                .field("path", path)
                .field("password", &"[hidden]")
                .finish(),
            SignerConfig::AwsKms { key } => f.debug_struct("AwsKms").field("key", key).finish(),
            SignerConfig::GoogleKms {
                project_id,
                location,
                key_ring,
                key,
                version,
            } => f
                .debug_struct("GoogleKms")
                .field("project_id", project_id)
                .field("location", location)
                .field("key_ring", key_ring)
                .field("key", key)
                .field("version", version)
                .finish(),
            SignerConfig::AlicloudKms { key, secret: _ } => f
                .debug_struct("AlicloudKms")
                .field("key", key)
                .field("secret", &"[hidden]")
                .finish(),
            SignerConfig::AzureKeyVault { key, secret: _ } => f
                .debug_struct("AzureKeyVault")
                .field("key", key)
                .field("secret", &"[hidden]")
                .finish(),
        }
    }
}
