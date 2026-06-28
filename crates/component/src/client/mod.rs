//! The stateful client lanes: the shared quote→sign→post lifecycle and the
//! keys-out host adapters, the orderbook read/write dispatch, and the two world
//! bindings — synchronous (WASI 0.2) and asynchronous (WASI 0.3) — over the SDK
//! `HttpTransport` seam.

// Shared lifecycle (quote → sign → post → allowance) and the keys-out
// `HostSigner` / `ContractReadProvider` adapters, generic over the transport.
mod core;
// Orderbook read + write dispatch over the lane's transport, returning canonical
// JSON or a typed `ReadError`.
mod orderbook;

// ===== world: client-sync (WASI 0.2, synchronous exports over wstd) =====
#[cfg(all(
    target_arch = "wasm32",
    feature = "world-client-sync",
    not(any(feature = "world-engine", feature = "world-client-async"))
))]
#[allow(
    unsafe_code,
    clippy::same_length_and_capacity,
    reason = "wit-bindgen's export! generates the Canonical ABI glue (#[export_name], unsafe, and raw Vec reconstruction); this crate writes none of its own"
)]
mod sync;

// ===== world: client-async (WASI 0.3, asynchronous exports over wasip3) =====
#[cfg(all(
    target_arch = "wasm32",
    feature = "world-client-async",
    not(any(feature = "world-engine", feature = "world-client-sync"))
))]
#[allow(
    unsafe_code,
    clippy::same_length_and_capacity,
    reason = "wit-bindgen's export! generates the Canonical ABI glue (#[export_name], unsafe, and raw Vec reconstruction); this crate writes none of its own"
)]
mod r#async;
