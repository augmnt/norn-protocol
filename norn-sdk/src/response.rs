//! Structured response type and helpers for contract results.
//!
//! The [`Response`] type replaces raw `Vec<u8>` as the success value in
//! [`ContractResult`]. It carries optional binary data plus key-value
//! attributes that are emitted as log messages on-chain.

use alloc::string::String;
use alloc::vec::Vec;
use borsh::BorshSerialize;

use crate::error::ContractError;

/// The result type returned by contract `execute` and `query` methods.
pub type ContractResult = Result<Response, ContractError>;

/// A key-value attribute included in a contract response.
///
/// Attributes are emitted as log messages via the host when the response
/// is returned from `execute` or `query`.
#[derive(Debug, Clone)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

/// A structured event emitted by a contract.
///
/// Events have a type name and a list of key-value attributes, similar to
/// Solidity's indexed events. They are emitted via the `norn_emit_event`
/// host function and appear in execution/query results.
///
/// ```ignore
/// Ok(Response::new()
///     .add_event(Event::new("Transfer")
///         .add_attribute("from", addr_to_hex(&sender))
///         .add_attribute("to", addr_to_hex(&to))
///         .add_attribute("amount", format!("{amount}"))))
/// ```
#[derive(Debug, Clone)]
pub struct Event {
    /// Event type name.
    pub ty: String,
    /// Key-value attributes.
    pub attributes: Vec<Attribute>,
}

impl Event {
    /// Create a new event with the given type name.
    pub fn new(ty: impl Into<String>) -> Self {
        Event {
            ty: ty.into(),
            attributes: Vec::new(),
        }
    }

    /// Add a key-value attribute to the event.
    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
        self
    }
}

/// Structured contract response with optional data, key-value attributes,
/// and structured events.
///
/// Use the builder pattern to construct responses:
///
/// ```ignore
/// Ok(Response::new()
///     .add_attribute("action", "transfer")
///     .add_event(Event::new("Transfer")
///         .add_attribute("from", "0x...")
///         .add_attribute("amount", "1000"))
///     .set_data(&balance))
/// ```
#[derive(Debug)]
pub struct Response {
    data: Vec<u8>,
    attributes: Vec<Attribute>,
    events: Vec<Event>,
}

impl Response {
    /// Create an empty response.
    pub fn new() -> Self {
        Response {
            data: Vec::new(),
            attributes: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Set the response data by borsh-serializing a value.
    pub fn set_data<T: BorshSerialize>(mut self, value: &T) -> Self {
        if let Ok(bytes) = borsh::to_vec(value) {
            self.data = bytes;
        }
        self
    }

    /// Set the response data from raw bytes.
    pub fn set_data_raw(mut self, bytes: Vec<u8>) -> Self {
        self.data = bytes;
        self
    }

    /// Add a key-value attribute to the response.
    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Add a structured event to the response.
    pub fn add_event(mut self, event: Event) -> Self {
        self.events.push(event);
        self
    }

    /// Get the response data bytes.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the response attributes.
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    /// Get the response events.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Emit attributes as log messages and events via the host.
    #[doc(hidden)]
    pub fn __emit_to_host(&self) {
        for attr in &self.attributes {
            let msg = alloc::format!("{}={}", attr.key, attr.value);
            crate::host::log(&msg);
        }
        for event in &self.events {
            crate::host::emit_event(&event.ty, &event.attributes);
        }
    }

    /// Get the data bytes (used by norn_entry! macro).
    #[doc(hidden)]
    pub fn __data(&self) -> &[u8] {
        &self.data
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

/// Borsh-serialize a value and return it as a successful [`Response`].
pub fn ok<T: BorshSerialize>(value: T) -> ContractResult {
    let data = borsh::to_vec(&value)
        .map_err(|e| ContractError::Custom(alloc::format!("serialize: {e}")))?;
    Ok(Response {
        data,
        attributes: Vec::new(),
        events: Vec::new(),
    })
}

/// Return raw bytes as a successful [`Response`].
pub fn ok_bytes(data: &[u8]) -> ContractResult {
    Ok(Response {
        data: data.to_vec(),
        attributes: Vec::new(),
        events: Vec::new(),
    })
}

/// Return an empty successful [`Response`].
pub fn ok_empty() -> ContractResult {
    Ok(Response::new())
}
