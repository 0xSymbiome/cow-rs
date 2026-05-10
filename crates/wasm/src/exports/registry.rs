use std::{
    cell::RefCell,
    collections::HashMap,
    sync::atomic::{AtomicU32, Ordering},
};

use js_sys::Function;
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

use crate::exports::errors::WasmError;

const CALLBACK_KEY_RESERVED_INVALID: u32 = 0;
const MAX_ALLOCATION_ATTEMPTS: u32 = 16;

type FetchAdapterClosure = Closure<dyn FnMut(JsValue) -> JsValue>;

thread_local! {
    static FETCH_CALLBACKS: RefCell<HashMap<FetchCallbackKey, CallbackEntry>> =
        RefCell::new(HashMap::new());
}

static NEXT_CALLBACK_KEY: AtomicU32 = AtomicU32::new(1);

/// Registry key for a callback transport fetch function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct FetchCallbackKey(u32);

impl FetchCallbackKey {
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}

struct CallbackEntry {
    callback: Function,
    _adapter: Option<FetchAdapterClosure>,
}

/// RAII registration guard for callback transports.
pub(crate) struct FetchCallbackGuard {
    id: FetchCallbackKey,
}

impl FetchCallbackGuard {
    pub(crate) const fn id(&self) -> FetchCallbackKey {
        self.id
    }
}

impl Drop for FetchCallbackGuard {
    fn drop(&mut self) {
        unregister_fetch_callback(self.id);
    }
}

pub(crate) fn register_fetch_callback(callback: Function) -> Result<FetchCallbackGuard, JsValue> {
    register_callback_entry(CallbackEntry {
        callback,
        _adapter: None,
    })
}

pub(crate) fn register_fetch_adapter(fetch: Function) -> Result<FetchCallbackGuard, JsValue> {
    let adapter = Closure::wrap(Box::new(move |request: JsValue| -> JsValue {
        crate::exports::transport::dispatch_fetch_adapter(&fetch, request)
    }) as Box<dyn FnMut(JsValue) -> JsValue>);
    let callback = adapter.as_ref().unchecked_ref::<Function>().clone();
    register_callback_entry(CallbackEntry {
        callback,
        _adapter: Some(adapter),
    })
}

pub(crate) fn lookup_fetch_callback(id: FetchCallbackKey) -> Option<Function> {
    FETCH_CALLBACKS.with(|cell| cell.borrow().get(&id).map(|entry| entry.callback.clone()))
}

fn register_callback_entry(entry: CallbackEntry) -> Result<FetchCallbackGuard, JsValue> {
    let id = allocate_callback_key()?;
    FETCH_CALLBACKS.with(|cell| {
        cell.borrow_mut().insert(id, entry);
    });
    Ok(FetchCallbackGuard { id })
}

fn unregister_fetch_callback(id: FetchCallbackKey) {
    FETCH_CALLBACKS.with(|cell| {
        cell.borrow_mut().remove(&id);
    });
}

fn allocate_callback_key() -> Result<FetchCallbackKey, JsValue> {
    for _ in 0..MAX_ALLOCATION_ATTEMPTS {
        let raw = NEXT_CALLBACK_KEY.fetch_add(1, Ordering::Relaxed);
        let id = if raw == CALLBACK_KEY_RESERVED_INVALID {
            NEXT_CALLBACK_KEY.fetch_add(1, Ordering::Relaxed)
        } else {
            raw
        };
        if id == CALLBACK_KEY_RESERVED_INVALID {
            continue;
        }
        let callback_key = FetchCallbackKey(id);
        let collision = FETCH_CALLBACKS.with(|cell| cell.borrow().contains_key(&callback_key));
        if !collision {
            return Ok(callback_key);
        }
    }

    Err(WasmError::internal("fetch callback registry key space exhausted").into_js())
}
