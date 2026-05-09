use cow_sdk_signing::GeneratedOrderId;

use crate::pure::dto::GeneratedOrderUidDto;

/// Converts generated UID data into canonical string DTO fields.
#[must_use]
pub fn generated_order_uid_dto(generated: &GeneratedOrderId) -> GeneratedOrderUidDto {
    GeneratedOrderUidDto {
        order_uid: generated.order_id.as_str().to_owned(),
        order_digest: generated.order_digest.as_str().to_owned(),
    }
}
