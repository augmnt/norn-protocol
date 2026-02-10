//! Structured response type and helpers for contract results.
//!
//! The [`Response`] type replaces raw `Vec<u8>` as the success value in
//! [`ContractResult`]. It carries optional binary data plus key-value
//! attributes that are emitted as log messages on-chain.

use alloc::string::String;
use alloc::vec::Vec;
use borsh::BorshSerialize;

use crate::error::ContractError;
use crate::types::Address;

/// Trait for types that can be converted to attribute string values.
///
/// Implemented for common contract types to enable ergonomic attribute building
/// on [`Response`] and [`Event`].
pub trait ToAttributeValue {
    /// Convert to a string suitable for use as an attribute value.
    fn to_attribute_value(&self) -> String;
}

impl ToAttributeValue for &str {
    fn to_attribute_value(&self) -> String {
        String::from(*self)
    }
}

impl ToAttributeValue for String {
    fn to_attribute_value(&self) -> String {
        self.clone()
    }
}

impl ToAttributeValue for u128 {
    fn to_attribute_value(&self) -> String {
        alloc::format!("{self}")
    }
}

impl ToAttributeValue for u64 {
    fn to_attribute_value(&self) -> String {
        alloc::format!("{self}")
    }
}

impl ToAttributeValue for Address {
    fn to_attribute_value(&self) -> String {
        crate::addr::addr_to_hex(self)
    }
}

impl ToAttributeValue for &Address {
    fn to_attribute_value(&self) -> String {
        crate::addr::addr_to_hex(self)
    }
}

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

    /// Add an address attribute (auto-converts to hex string).
    pub fn add_address(self, key: impl Into<String>, addr: &Address) -> Self {
        self.add_attribute(key, crate::addr::addr_to_hex(addr))
    }

    /// Add a u128 attribute (auto-converts to decimal string).
    pub fn add_u128(self, key: impl Into<String>, value: u128) -> Self {
        self.add_attribute(key, alloc::format!("{value}"))
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

    /// Create a response with an "action" attribute pre-set.
    pub fn with_action(action: impl Into<String>) -> Self {
        Self::new().add_attribute("action", action)
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

    /// Add an address attribute (auto-converts to hex string).
    pub fn add_address(self, key: impl Into<String>, addr: &Address) -> Self {
        self.add_attribute(key, crate::addr::addr_to_hex(addr))
    }

    /// Add a u128 attribute (auto-converts to decimal string).
    pub fn add_u128(self, key: impl Into<String>, value: u128) -> Self {
        self.add_attribute(key, alloc::format!("{value}"))
    }

    /// Add a structured event to the response.
    pub fn add_event(mut self, event: Event) -> Self {
        self.events.push(event);
        self
    }

    /// Merge another response into this one.
    ///
    /// Appends the other response's attributes and events. If the other
    /// response has data and this one doesn't, adopts the other's data.
    /// This enables composing stdlib responses with contract-specific attributes:
    ///
    /// ```ignore
    /// let stdlib_resp = Norn20::mint(&to, amount)?;
    /// Ok(Response::with_action("mint").merge(stdlib_resp))
    /// ```
    pub fn merge(mut self, other: Response) -> Self {
        self.attributes.extend(other.attributes);
        self.events.extend(other.events);
        if self.data.is_empty() && !other.data.is_empty() {
            self.data = other.data;
        }
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
