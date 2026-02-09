use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

/// Handle `#[norn_contract]` on a struct definition.
///
/// Adds `#[derive(BorshSerialize, BorshDeserialize)]` automatically.
pub fn expand(item: ItemStruct) -> TokenStream {
    let vis = &item.vis;
    let ident = &item.ident;
    let generics = &item.generics;
    let attrs = &item.attrs;
    let fields = &item.fields;
    let semi = &item.semi_token;

    // Re-emit the struct with borsh derives prepended.
    match fields {
        syn::Fields::Named(_) => {
            quote! {
                #(#attrs)*
                #[derive(::borsh::BorshSerialize, ::borsh::BorshDeserialize)]
                #vis struct #ident #generics #fields
            }
        }
        syn::Fields::Unnamed(_) => {
            quote! {
                #(#attrs)*
                #[derive(::borsh::BorshSerialize, ::borsh::BorshDeserialize)]
                #vis struct #ident #generics #fields #semi
            }
        }
        syn::Fields::Unit => {
            quote! {
                #(#attrs)*
                #[derive(::borsh::BorshSerialize, ::borsh::BorshDeserialize)]
                #vis struct #ident #generics ;
            }
        }
    }
}
