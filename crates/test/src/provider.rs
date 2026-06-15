//! [`MockProvider`]: an in-memory [`Provider`] + [`SigningProvider`] double that
//! returns canned chain-RPC values and records contract reads and calls.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use cow_sdk_core::{
    Address, Amount, BlockInfo, ContractCall, HexData, Provider, SigningProvider, SupportedChainId,
    TransactionHash, TransactionReceipt, TransactionRequest,
};

use crate::{error::MockError, signer::MockSigner};

/// A recording, canned-response [`Provider`] and [`SigningProvider`] double.
///
/// Its `Error` is [`MockError`], matching [`MockSigner`] so the
/// `SigningProvider::Signer: Signer<Error = Self::Error>` bound holds.
#[derive(Clone, Debug)]
pub struct MockProvider {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Debug)]
struct Inner {
    chain_id: SupportedChainId,
    allowance: Amount,
    code: Option<HexData>,
    receipt: Option<TransactionReceipt>,
    receipt_sequence: VecDeque<Option<TransactionReceipt>>,
    signer: MockSigner,
    fail_call: Option<String>,
    calls: ProviderCalls,
}

/// A snapshot of what a [`MockProvider`] was asked to do.
#[derive(Clone, Debug, Default)]
pub struct ProviderCalls {
    /// Contract reads passed to [`Provider::read_contract`].
    pub contract_reads: Vec<ContractCall>,
    /// Transactions passed to [`Provider::call`].
    pub calls: Vec<TransactionRequest>,
}

impl MockProvider {
    /// A provider with canned defaults and a default [`MockSigner`].
    #[must_use]
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Starts a builder to configure canned values and injected failures.
    #[must_use]
    pub fn builder() -> MockProviderBuilder {
        MockProviderBuilder::default()
    }

    /// A snapshot of the calls recorded so far.
    #[must_use]
    pub fn recorded(&self) -> ProviderCalls {
        self.lock().calls.clone()
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Consuming builder for [`MockProvider`].
#[derive(Clone, Debug)]
pub struct MockProviderBuilder {
    chain_id: SupportedChainId,
    allowance: Amount,
    code: Option<HexData>,
    receipt: Option<TransactionReceipt>,
    receipt_sequence: VecDeque<Option<TransactionReceipt>>,
    signer: MockSigner,
    fail_call: Option<String>,
}

impl Default for MockProviderBuilder {
    fn default() -> Self {
        Self {
            chain_id: SupportedChainId::Mainnet,
            allowance: Amount::from(1_000_000_000_000_000_000_u64),
            code: None,
            receipt: None,
            receipt_sequence: VecDeque::new(),
            signer: MockSigner::new(),
            fail_call: None,
        }
    }
}

impl MockProviderBuilder {
    /// Sets the chain id [`Provider::get_chain_id`] reports.
    #[must_use]
    pub const fn chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Sets the allowance [`Provider::read_contract`] returns (its decimal
    /// atoms are the canned `eth_call`-style read result).
    #[must_use]
    pub const fn allowance(mut self, allowance: Amount) -> Self {
        self.allowance = allowance;
        self
    }

    /// Sets the code [`Provider::get_code`] returns.
    #[must_use]
    pub fn code(mut self, code: HexData) -> Self {
        self.code = Some(code);
        self
    }

    /// Sets the single receipt [`Provider::get_transaction_receipt`] returns once
    /// any [`MockProviderBuilder::receipt_sequence`] script is exhausted.
    #[must_use]
    pub const fn transaction_receipt(mut self, receipt: TransactionReceipt) -> Self {
        self.receipt = Some(receipt);
        self
    }

    /// Scripts successive [`Provider::get_transaction_receipt`] outcomes, one entry
    /// consumed per call: `None` models a transaction that is not mined yet, and
    /// `Some(receipt)` a mined result. Once the script is exhausted the provider
    /// falls back to [`MockProviderBuilder::transaction_receipt`] (absent by
    /// default, modelling a transaction that never appears).
    ///
    /// This lets a consumer drive a receipt-polling wait — a receipt that arrives
    /// after a number of polls, a reverted receipt, or a timeout — without a live
    /// endpoint.
    #[must_use]
    pub fn receipt_sequence(
        mut self,
        receipts: impl IntoIterator<Item = Option<TransactionReceipt>>,
    ) -> Self {
        self.receipt_sequence = receipts.into_iter().collect();
        self
    }

    /// Sets the signer [`SigningProvider::create_signer`] returns.
    #[must_use]
    pub fn signer(mut self, signer: MockSigner) -> Self {
        self.signer = signer;
        self
    }

    /// Makes [`Provider::call`] fail with `error`.
    #[must_use]
    pub fn fail_call(mut self, error: impl Into<String>) -> Self {
        self.fail_call = Some(error.into());
        self
    }

    /// Builds the provider.
    #[must_use]
    pub fn build(self) -> MockProvider {
        MockProvider {
            inner: Arc::new(Mutex::new(Inner {
                chain_id: self.chain_id,
                allowance: self.allowance,
                code: self.code,
                receipt: self.receipt,
                receipt_sequence: self.receipt_sequence,
                signer: self.signer,
                fail_call: self.fail_call,
                calls: ProviderCalls::default(),
            })),
        }
    }
}

impl Provider for MockProvider {
    type Error = MockError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(u64::from(self.lock().chain_id))
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(self.lock().code.clone())
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        let mut guard = self.lock();
        if let Some(scripted) = guard.receipt_sequence.pop_front() {
            return Ok(scripted);
        }
        Ok(guard.receipt.clone())
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        let mut guard = self.lock();
        if let Some(error) = &guard.fail_call {
            return Err(MockError::new(error.clone()));
        }
        guard.calls.calls.push(tx.clone());
        drop(guard);
        Ok(HexData::empty())
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        let mut guard = self.lock();
        guard.calls.contract_reads.push(request.clone());
        Ok(guard.allowance.to_string())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }
}

impl SigningProvider for MockProvider {
    type Signer = MockSigner;

    async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.lock().signer.clone())
    }
}
