use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
};

use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::exports::errors::WasmError;

/// Reserved invalid fetch-callback handle id.
pub const HANDLE_ID_RESERVED_INVALID: u32 = 0;

const MAX_ALLOCATION_ATTEMPTS: u32 = 16;

thread_local! {
    static FETCH_CALLBACKS: RefCell<HashMap<FetchCallbackHandleId, Function>> =
        RefCell::new(HashMap::new());
}

static NEXT_HANDLE_ID: AtomicU32 = AtomicU32::new(1);

/// Registry id for a callback transport fetch function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct FetchCallbackHandleId(pub u32);

impl FetchCallbackHandleId {
    /// Creates a handle id from a raw JS-visible integer.
    ///
    /// # Errors
    ///
    /// Returns a JS error when `id` is the reserved zero sentinel.
    pub fn new(id: u32) -> Result<Self, JsValue> {
        if id == HANDLE_ID_RESERVED_INVALID {
            return Err(WasmError::invalid(
                "fetchCallbackId",
                "fetch callback handle id 0 is reserved as invalid",
            )
            .into_js());
        }
        Ok(Self(id))
    }
}

/// Disposable callback registry handle.
#[wasm_bindgen]
pub struct FetchCallbackHandle {
    id: FetchCallbackHandleId,
    disposed: Cell<bool>,
}

#[wasm_bindgen]
impl FetchCallbackHandle {
    /// Numeric callback id.
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id.0
    }

    /// Disposes this callback registration. Calling this more than once is harmless.
    pub fn dispose(&self) {
        if self.disposed.replace(true) {
            return;
        }
        unregister_fetch_callback(self.id);
    }
}

impl FetchCallbackHandle {
    pub(crate) const fn handle_id(&self) -> FetchCallbackHandleId {
        self.id
    }
}

impl Drop for FetchCallbackHandle {
    fn drop(&mut self) {
        if !self.disposed.replace(true) {
            unregister_fetch_callback(self.id);
        }
    }
}

/// Registers a JS fetch callback and returns a disposable handle.
#[wasm_bindgen(js_name = "registerFetchCallback")]
pub fn register_fetch_callback(callback: Function) -> Result<FetchCallbackHandle, JsValue> {
    let id = allocate_handle_id()?;
    FETCH_CALLBACKS.with(|cell| {
        cell.borrow_mut().insert(id, callback);
    });
    Ok(FetchCallbackHandle {
        id,
        disposed: Cell::new(false),
    })
}

pub(crate) fn lookup_fetch_callback(id: FetchCallbackHandleId) -> Option<Function> {
    FETCH_CALLBACKS.with(|cell| cell.borrow().get(&id).cloned())
}

fn unregister_fetch_callback(id: FetchCallbackHandleId) {
    FETCH_CALLBACKS.with(|cell| {
        cell.borrow_mut().remove(&id);
    });
}

fn allocate_handle_id() -> Result<FetchCallbackHandleId, JsValue> {
    for _ in 0..MAX_ALLOCATION_ATTEMPTS {
        let raw = NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed);
        let id = if raw == HANDLE_ID_RESERVED_INVALID {
            NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed)
        } else {
            raw
        };
        if id == HANDLE_ID_RESERVED_INVALID {
            continue;
        }
        let handle_id = FetchCallbackHandleId(id);
        let collision = FETCH_CALLBACKS.with(|cell| cell.borrow().contains_key(&handle_id));
        if !collision {
            return Ok(handle_id);
        }
    }

    Err(WasmError::Internal {
        message: "fetch callback handle space exhausted".to_owned(),
    }
    .into_js())
}
