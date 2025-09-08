mod config;
pub use config::SignerConfig;

use crate::prelude::*;
use alloy::{
    network::{EthereumWallet, TxSigner},
    primitives::{Address, Signature},
    signers::{
        aws::{AwsSigner, aws_config, aws_sdk_kms},
        gcp::{
            GcpKeyRingRef, GcpSigner, KeySpecifier,
            gcloud_sdk::{
                GoogleApi,
                google::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient,
            },
        },
        local::{LocalSigner, MnemonicBuilder, PrivateKeySigner, coins_bip39::English},
    },
};

impl SignerConfig {
    async fn signer(&self) -> Result<Box<dyn TxSigner<Signature> + Send + Sync + 'static>> {
        let signer: Box<dyn TxSigner<Signature> + Send + Sync + 'static> = match self {
            SignerConfig::PrivateKey(key) => Box::new(key.parse::<PrivateKeySigner>()?),
            SignerConfig::Mnemonic(mnemonic) => Box::new(
                MnemonicBuilder::<English>::default()
                    .phrase(mnemonic)
                    .build()?,
            ),
            SignerConfig::KeyStore { path, password } => {
                Box::new(LocalSigner::decrypt_keystore(path, password)?)
            }
            SignerConfig::AwsKms { key } => {
                let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
                let client = aws_sdk_kms::Client::new(&config);
                Box::new(
                    AwsSigner::new(client, key.clone(), Some(1))
                        .await
                        .map_err(Box::new)?,
                )
            }
            Self::GoogleKms {
                project_id,
                location,
                key_ring,
                key,
                version,
            } => {
                let keyring_ref = GcpKeyRingRef::new(project_id, location, key_ring);

                let client = GoogleApi::from_function(
                    KeyManagementServiceClient::new,
                    "https://cloudkms.googleapis.com",
                    None,
                )
                .await?;
                let key_specifier = KeySpecifier::new(keyring_ref, key, *version);

                Box::new(
                    GcpSigner::new(client, key_specifier, None)
                        .await
                        .map_err(Box::new)?,
                )
            }
            _ => unimplemented!(),
        };
        Ok(signer)
    }

    pub async fn wallet(&self) -> Result<EthereumWallet> {
        let signer = self.signer().await?;
        Ok(EthereumWallet::new(signer))
    }

    pub async fn address(&self) -> Result<Address> {
        let signer = self.signer().await?;
        Ok(signer.address())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn parase_private_key() {
        let s = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";
        let signer = s.parse::<PrivateKeySigner>().unwrap();
        assert_eq!(
            format!("{}", signer.address()),
            "0xbb48b4d059D901F0CE1325d1A37f9E14C6634499"
        );
    }

    #[test]
    fn parse_mnemonic() {
        let s = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let signer = MnemonicBuilder::<English>::default()
            .phrase(s)
            .build()
            .unwrap();
        assert_eq!(
            format!("{}", signer.address()),
            "0x9858EfFD232B4033E47d90003D41EC34EcaEda94"
        );
    }
}
