use cow_sdk_core::{Address, AsyncSigningProvider};

use crate::{BrowserWalletError, signer::Eip1193Signer};

use super::Eip1193Provider;

#[allow(async_fn_in_trait)]
impl AsyncSigningProvider for Eip1193Provider {
    type Signer = Eip1193Signer;

    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        let account_hint = if signer_hint.trim().is_empty() {
            None
        } else {
            Some(Address::new(signer_hint.trim())?)
        };
        if let Some(expected) = &account_hint {
            let accounts = if self.session.borrow().accounts.is_empty() {
                self.query_accounts(false).await?
            } else {
                self.session.borrow().accounts.clone()
            };
            if !accounts
                .iter()
                .any(|candidate| candidate.to_hex_string() == expected.to_hex_string())
            {
                return Err(BrowserWalletError::malformed_response(
                    "create_signer",
                    format!(
                        "wallet does not expose account {}",
                        expected.to_hex_string()
                    ),
                ));
            }
        }
        Ok(Eip1193Signer::new(self.clone(), account_hint))
    }
}
