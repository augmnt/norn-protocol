//! Proc macros for the Norn SDK.
//!
//! Provides `#[norn_contract]` — an attribute macro that eliminates ceremony
//! from loom smart contract definitions.

mod contract_impl;
mod contract_struct;
mod util;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Item};

/// Attribute macro for Norn loom smart contracts.
///
/// # On a struct
///
/// Automatically adds `#[derive(BorshSerialize, BorshDeserialize)]`.
///
/// ```ignore
/// #[norn_contract]
/// pub struct Counter { value: u64 }
/// // expands to:
/// #[derive(BorshSerialize, BorshDeserialize)]
/// pub struct Counter { value: u64 }
/// ```
///
/// # On an impl block
///
/// Generates dispatch enums, `Contract` trait impl, and `norn_entry!` call
/// from annotated methods:
///
/// - `#[init]` — constructor (exactly one required, must return `Self`)
/// - `#[execute]` — state-changing operation (`&mut self, &Context, ...`)
/// - `#[query]` — read-only operation (`&self, &Context, ...`)
/// - Unmarked methods are kept as internal helpers.
///
/// ```ignore
/// #[norn_contract]
/// impl Counter {
///     #[init]
///     pub fn new(_ctx: &Context) -> Self { Counter { value: 0 } }
///
///     #[execute]
///     pub fn increment(&mut self, _ctx: &Context) -> ContractResult {
///         self.value += 1;
///         ok(self.value)
///     }
///
///     #[query]
///     pub fn get_value(&self, _ctx: &Context) -> ContractResult {
///         ok(self.value)
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn norn_contract(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // Try to parse as a general Item to determine if it's a struct or impl.
    let item = parse_macro_input!(input as Item);
    match item {
        Item::Struct(s) => contract_struct::expand(s).into(),
        Item::Impl(i) => contract_impl::expand(i).into(),
        _ => syn::Error::new_spanned(
            proc_macro2::TokenStream::new(),
            "#[norn_contract] can only be applied to a struct or impl block",
        )
        .to_compile_error()
        .into(),
    }
}
