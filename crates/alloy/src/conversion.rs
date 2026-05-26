//! Conversion helpers for the native composed Alloy adapter.
//!
//! This module is a thin re-export shim over the leaf adapters' inter-crate
//! seam modules. The provider leaf owns the JSON-RPC request, block-tag,
//! receipt, and block-info conversions; the signer leaf owns the EIP-712
//! typed-data conversion and signature normalization. Both surfaces are
//! consumed through their respective doc-hidden seam modules so the
//! umbrella does not maintain a parallel copy of either implementation.

pub(crate) use cow_sdk_alloy_provider::__seam::{
    alloy_to_cow_block_info, alloy_to_cow_receipt, cow_block_tag_to_alloy, cow_request_to_alloy,
    rpc_error_to_class_and_detail,
};

pub(crate) use cow_sdk_alloy_signer::__seam::{
    alloy_signature_to_hex, cow_flat_to_alloy_typed_data, cow_typed_data_payload_to_alloy,
};
