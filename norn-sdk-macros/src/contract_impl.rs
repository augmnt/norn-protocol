use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{ImplItem, ImplItemFn, ItemImpl, ReturnType, Type};

use crate::util::{extract_params, snake_to_pascal, ExtractedParam};

/// A parsed method with its extracted parameters.
struct MethodInfo {
    method: ImplItemFn,
    params: Vec<ExtractedParam>,
}

enum MethodRole {
    Init,
    Execute,
    Query,
}

/// Handle `#[norn_contract]` on an `impl` block.
///
/// Scans for `#[init]`, `#[execute]`, `#[query]` attributes on methods, then
/// generates the Execute/Query enums, Contract trait impl, and norn_entry! call.
pub fn expand(item: ItemImpl) -> TokenStream {
    let struct_ty = &item.self_ty;

    // Extract the struct name for generated type names.
    let struct_name = match struct_ty.as_ref() {
        Type::Path(tp) => {
            if let Some(seg) = tp.path.segments.last() {
                seg.ident.clone()
            } else {
                return syn::Error::new_spanned(struct_ty, "expected a struct name")
                    .to_compile_error();
            }
        }
        _ => {
            return syn::Error::new_spanned(struct_ty, "expected a struct name").to_compile_error();
        }
    };

    // Parse all methods.
    let mut init_method: Option<MethodInfo> = None;
    let mut execute_methods: Vec<MethodInfo> = Vec::new();
    let mut query_methods: Vec<MethodInfo> = Vec::new();
    let mut helper_items: Vec<ImplItem> = Vec::new();

    for item in item.items.iter() {
        match item {
            ImplItem::Fn(method) => {
                let role = detect_role(method);
                match role {
                    Some(MethodRole::Init) => {
                        if init_method.is_some() {
                            return syn::Error::new_spanned(
                                &method.sig.ident,
                                "only one #[init] method is allowed",
                            )
                            .to_compile_error();
                        }
                        // Validate: must return Self
                        if !returns_self(&method.sig.output) {
                            return syn::Error::new_spanned(
                                &method.sig.ident,
                                "#[init] method must return Self",
                            )
                            .to_compile_error();
                        }
                        let params = extract_params(&method.sig.inputs);
                        init_method = Some(MethodInfo {
                            method: strip_markers(method.clone()),
                            params,
                        });
                    }
                    Some(MethodRole::Execute) => {
                        // Validate: must have &mut self
                        if !has_mut_self(method) {
                            return syn::Error::new_spanned(
                                &method.sig.ident,
                                "#[execute] method must take &mut self",
                            )
                            .to_compile_error();
                        }
                        if !has_context_param(method) {
                            return syn::Error::new_spanned(
                                &method.sig.ident,
                                "#[execute] method must take &Context as second parameter",
                            )
                            .to_compile_error();
                        }
                        let params = extract_params(&method.sig.inputs);
                        execute_methods.push(MethodInfo {
                            method: strip_markers(method.clone()),
                            params,
                        });
                    }
                    Some(MethodRole::Query) => {
                        // Validate: must have &self
                        if !has_ref_self(method) {
                            return syn::Error::new_spanned(
                                &method.sig.ident,
                                "#[query] method must take &self",
                            )
                            .to_compile_error();
                        }
                        if !has_context_param(method) {
                            return syn::Error::new_spanned(
                                &method.sig.ident,
                                "#[query] method must take &Context as second parameter",
                            )
                            .to_compile_error();
                        }
                        let params = extract_params(&method.sig.inputs);
                        query_methods.push(MethodInfo {
                            method: strip_markers(method.clone()),
                            params,
                        });
                    }
                    None => {
                        // Internal helper — keep as-is.
                        helper_items.push(ImplItem::Fn(method.clone()));
                    }
                }
            }
            other => {
                helper_items.push(other.clone());
            }
        }
    }

    // Validate: at least one #[init] method.
    let init = match init_method {
        Some(m) => m,
        None => {
            return syn::Error::new(
                Span::call_site(),
                "#[norn_contract] impl block must have exactly one #[init] method",
            )
            .to_compile_error();
        }
    };

    // Generate names.
    let exec_enum_name = format_ident!("__{}Execute", struct_name);
    let query_enum_name = format_ident!("__{}Query", struct_name);

    // Generate execute enum.
    let exec_enum = generate_enum(&exec_enum_name, &execute_methods);

    // Generate query enum.
    let query_enum = generate_enum(&query_enum_name, &query_methods);

    // Generate init type.
    let (init_type, init_struct_def) = generate_init_type(&struct_name, &init);

    // Generate Contract trait impl.
    let contract_impl = generate_contract_impl(
        &struct_name,
        &init,
        &execute_methods,
        &query_methods,
        &exec_enum_name,
        &query_enum_name,
        &init_type,
    );

    // Collect all cleaned methods for the user's impl block.
    let mut all_methods: Vec<&ImplItemFn> = Vec::new();
    all_methods.push(&init.method);
    for m in &execute_methods {
        all_methods.push(&m.method);
    }
    for m in &query_methods {
        all_methods.push(&m.method);
    }

    // Re-emit the impl block with cleaned methods + helpers.
    let impl_attrs = &item.attrs;
    let generics = &item.generics;

    quote! {
        #init_struct_def

        #[derive(::borsh::BorshSerialize, ::borsh::BorshDeserialize)]
        #exec_enum

        #[derive(::borsh::BorshSerialize, ::borsh::BorshDeserialize)]
        #query_enum

        #(#impl_attrs)*
        impl #generics #struct_ty {
            #(#all_methods)*
            #(#helper_items)*
        }

        #contract_impl

        ::norn_sdk::norn_entry!(#struct_name);
    }
}

/// Detect the role of a method from its attributes.
fn detect_role(method: &ImplItemFn) -> Option<MethodRole> {
    for attr in &method.attrs {
        if attr.path().is_ident("init") {
            return Some(MethodRole::Init);
        }
        if attr.path().is_ident("execute") {
            return Some(MethodRole::Execute);
        }
        if attr.path().is_ident("query") {
            return Some(MethodRole::Query);
        }
    }
    None
}

/// Strip `#[init]`, `#[execute]`, `#[query]` attributes from a method.
fn strip_markers(mut method: ImplItemFn) -> ImplItemFn {
    method.attrs.retain(|attr| {
        !attr.path().is_ident("init")
            && !attr.path().is_ident("execute")
            && !attr.path().is_ident("query")
    });
    method
}

/// Check if a method has `&mut self`.
fn has_mut_self(method: &ImplItemFn) -> bool {
    method.sig.inputs.iter().any(|arg| {
        matches!(arg, syn::FnArg::Receiver(r) if r.mutability.is_some() && r.reference.is_some())
    })
}

/// Check if a method has `&self`.
fn has_ref_self(method: &ImplItemFn) -> bool {
    method
        .sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, syn::FnArg::Receiver(r) if r.reference.is_some()))
}

/// Check if a method has a `&Context` parameter.
fn has_context_param(method: &ImplItemFn) -> bool {
    method.sig.inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            is_context_type(&pat_type.ty)
        } else {
            false
        }
    })
}

fn is_context_type(ty: &Type) -> bool {
    match ty {
        Type::Reference(r) => is_context_type(&r.elem),
        Type::Path(tp) => tp
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Context"),
        _ => false,
    }
}

/// Check if the return type is `Self` or `-> Self`.
fn returns_self(output: &ReturnType) -> bool {
    match output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(tp) => tp.path.is_ident("Self"),
            _ => false,
        },
    }
}

/// Generate an enum from method infos.
fn generate_enum(name: &Ident, methods: &[MethodInfo]) -> TokenStream {
    if methods.is_empty() {
        // Empty enum — use a unit variant so borsh can derive.
        return quote! {
            #[doc(hidden)]
            pub enum #name {
                __Unused,
            }
        };
    }

    let variants: Vec<TokenStream> = methods
        .iter()
        .map(|m| {
            let variant_name = Ident::new(
                &snake_to_pascal(&m.method.sig.ident.to_string()),
                m.method.sig.ident.span(),
            );
            if m.params.is_empty() {
                quote! { #variant_name }
            } else {
                let fields: Vec<TokenStream> = m
                    .params
                    .iter()
                    .map(|p| {
                        let name = &p.name;
                        let ty = &p.ty;
                        quote! { #name: #ty }
                    })
                    .collect();
                quote! { #variant_name { #(#fields),* } }
            }
        })
        .collect();

    quote! {
        #[doc(hidden)]
        pub enum #name {
            #(#variants),*
        }
    }
}

/// Generate the init type. If init has extra params, generates a struct.
/// Otherwise, uses `::norn_sdk::types::Empty`.
fn generate_init_type(struct_name: &Ident, init: &MethodInfo) -> (TokenStream, TokenStream) {
    if init.params.is_empty() {
        (quote! { ::norn_sdk::types::Empty }, quote! {})
    } else {
        let init_struct_name = format_ident!("__{}Init", struct_name);
        let fields: Vec<TokenStream> = init
            .params
            .iter()
            .map(|p| {
                let name = &p.name;
                let ty = &p.ty;
                quote! { pub #name: #ty }
            })
            .collect();
        let def = quote! {
            #[doc(hidden)]
            #[derive(::borsh::BorshSerialize, ::borsh::BorshDeserialize)]
            pub struct #init_struct_name {
                #(#fields),*
            }
        };
        (quote! { #init_struct_name }, def)
    }
}

/// Generate the `Contract` trait impl.
fn generate_contract_impl(
    struct_name: &Ident,
    init: &MethodInfo,
    execute_methods: &[MethodInfo],
    query_methods: &[MethodInfo],
    exec_enum_name: &Ident,
    query_enum_name: &Ident,
    init_type: &TokenStream,
) -> TokenStream {
    // Init dispatch.
    let init_fn_name = &init.method.sig.ident;
    let init_body = if init.params.is_empty() {
        quote! {
            Self::#init_fn_name(__norn_ctx)
        }
    } else {
        let param_names: Vec<&Ident> = init.params.iter().map(|p| &p.name).collect();
        quote! {
            Self::#init_fn_name(__norn_ctx, #(__norn_init_msg.#param_names),*)
        }
    };

    let init_msg_param = if init.params.is_empty() {
        quote! { _msg: #init_type }
    } else {
        quote! { __norn_init_msg: #init_type }
    };

    // Execute dispatch.
    let exec_match_arms: Vec<TokenStream> = execute_methods
        .iter()
        .map(|m| {
            let variant_name =
                Ident::new(&snake_to_pascal(&m.method.sig.ident.to_string()), m.method.sig.ident.span());
            let fn_name = &m.method.sig.ident;
            if m.params.is_empty() {
                quote! {
                    #exec_enum_name::#variant_name => self.#fn_name(__norn_ctx)
                }
            } else {
                let destructure: Vec<&Ident> = m.params.iter().map(|p| &p.name).collect();
                let call_args: Vec<TokenStream> = m
                    .params
                    .iter()
                    .map(|p| {
                        let name = &p.name;
                        if p.is_ref {
                            quote! { &#name }
                        } else {
                            quote! { #name }
                        }
                    })
                    .collect();
                quote! {
                    #exec_enum_name::#variant_name { #(#destructure),* } => self.#fn_name(__norn_ctx, #(#call_args),*)
                }
            }
        })
        .collect();

    let exec_body = if exec_match_arms.is_empty() {
        quote! {
            match _msg {
                #exec_enum_name::__Unused => ::core::unreachable!(),
            }
        }
    } else {
        quote! {
            match __norn_msg {
                #(#exec_match_arms),*
            }
        }
    };

    let exec_msg_param = if execute_methods.is_empty() {
        quote! { _msg: #exec_enum_name }
    } else {
        quote! { __norn_msg: #exec_enum_name }
    };

    // Query dispatch.
    let query_match_arms: Vec<TokenStream> = query_methods
        .iter()
        .map(|m| {
            let variant_name =
                Ident::new(&snake_to_pascal(&m.method.sig.ident.to_string()), m.method.sig.ident.span());
            let fn_name = &m.method.sig.ident;
            if m.params.is_empty() {
                quote! {
                    #query_enum_name::#variant_name => self.#fn_name(__norn_ctx)
                }
            } else {
                let destructure: Vec<&Ident> = m.params.iter().map(|p| &p.name).collect();
                let call_args: Vec<TokenStream> = m
                    .params
                    .iter()
                    .map(|p| {
                        let name = &p.name;
                        if p.is_ref {
                            quote! { &#name }
                        } else {
                            quote! { #name }
                        }
                    })
                    .collect();
                quote! {
                    #query_enum_name::#variant_name { #(#destructure),* } => self.#fn_name(__norn_ctx, #(#call_args),*)
                }
            }
        })
        .collect();

    let query_body = if query_match_arms.is_empty() {
        quote! {
            match _msg {
                #query_enum_name::__Unused => ::core::unreachable!(),
            }
        }
    } else {
        quote! {
            match __norn_msg {
                #(#query_match_arms),*
            }
        }
    };

    let query_msg_param = if query_methods.is_empty() {
        quote! { _msg: #query_enum_name }
    } else {
        quote! { __norn_msg: #query_enum_name }
    };

    quote! {
        impl ::norn_sdk::Contract for #struct_name {
            type Init = #init_type;
            type Exec = #exec_enum_name;
            type Query = #query_enum_name;

            fn init(__norn_ctx: &::norn_sdk::Context, #init_msg_param) -> Self {
                #init_body
            }

            fn execute(&mut self, __norn_ctx: &::norn_sdk::Context, #exec_msg_param) -> ::norn_sdk::ContractResult {
                #exec_body
            }

            fn query(&self, __norn_ctx: &::norn_sdk::Context, #query_msg_param) -> ::norn_sdk::ContractResult {
                #query_body
            }
        }
    }
}
