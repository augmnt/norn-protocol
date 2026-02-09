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
#[derive(Debug)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

/// Structured contract response with optional data and key-value attributes.
///
/// Use the builder pattern to construct responses:
///
/// ```ignore
/// Ok(Response::new()
///     .add_attribute("action", "transfer")
///     .add_attribute("amount", "1000")
///     .set_data(&balance))
/// ```
#[derive(Debug)]
pub struct Response {
    data: Vec<u8>,
    attributes: Vec<Attribute>,
}

impl Response {
    /// Create an empty response.
    pub fn new() -> Self {
        Response {
            data: Vec::new(),
            attributes: Vec::new(),
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

    /// Get the response data bytes.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the response attributes.
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    /// Emit attributes as log messages via the host.
    #[doc(hidden)]
    pub fn __emit_to_host(&self) {
        for attr in &self.attributes {
            let msg = alloc::format!("{}={}", attr.key, attr.value);
            crate::host::log(&msg);
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
    })
}

/// Return raw bytes as a successful [`Response`].
pub fn ok_bytes(data: &[u8]) -> ContractResult {
    Ok(Response {
        data: data.to_vec(),
        attributes: Vec::new(),
    })
}

/// Return an empty successful [`Response`].
pub fn ok_empty() -> ContractResult {
    Ok(Response::new())
}
