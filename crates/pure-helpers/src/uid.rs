//! Order UID formatting helpers.

use cow_sdk_signing::GeneratedOrderId;

use crate::dto::GeneratedOrderUidDto;

/// Converts generated UID data into canonical string DTO fields.
#[must_use]
pub fn generated_order_uid_dto(generated: &GeneratedOrderId) -> GeneratedOrderUidDto {
    GeneratedOrderUidDto {
        order_uid: generated.order_id.to_hex_string(),
        order_digest: generated.order_digest.to_hex_string(),
    }
}
