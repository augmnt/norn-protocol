use proc_macro2::Ident;
use syn::{FnArg, Pat, PatType, Type};

/// Convert a `snake_case` identifier to `PascalCase`.
pub fn snake_to_pascal(s: &str) -> String {
    s.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let mut s = c.to_uppercase().to_string();
                    s.extend(chars);
                    s
                }
            }
        })
        .collect()
}

/// A parameter extracted from a method signature (after &self / &Context).
pub struct ExtractedParam {
    pub name: Ident,
    pub ty: Type,
    /// Whether the original type was a reference (`&T`). If so, `ty` stores
    /// the inner `T` and the dispatch call should pass `&var`.
    pub is_ref: bool,
}

/// Extract parameters from a method, skipping `self` and the `&Context` param.
/// Returns the list of "business" parameters.
pub fn extract_params(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
) -> Vec<ExtractedParam> {
    let mut params = Vec::new();
    for arg in inputs.iter() {
        match arg {
            FnArg::Receiver(_) => continue, // skip &self / &mut self
            FnArg::Typed(PatType { pat, ty, .. }) => {
                // Skip &Context parameters
                if is_context_type(ty) {
                    continue;
                }
                let name = match pat.as_ref() {
                    Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                    _ => continue,
                };
                let (inner_ty, is_ref) = strip_reference(ty);
                params.push(ExtractedParam {
                    name,
                    ty: inner_ty,
                    is_ref,
                });
            }
        }
    }
    params
}

/// Check if a type is `&Context` or `Context`.
fn is_context_type(ty: &Type) -> bool {
    match ty {
        Type::Reference(r) => is_context_type(&r.elem),
        Type::Path(tp) => {
            if let Some(seg) = tp.path.segments.last() {
                seg.ident == "Context"
            } else {
                false
            }
        }
        _ => false,
    }
}

/// If the type is `&T`, return `(T, true)`. Otherwise `(T, false)`.
fn strip_reference(ty: &Type) -> (Type, bool) {
    match ty {
        Type::Reference(r) => (*r.elem.clone(), true),
        other => (other.clone(), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_pascal() {
        assert_eq!(snake_to_pascal("increment"), "Increment");
        assert_eq!(snake_to_pascal("get_value"), "GetValue");
        assert_eq!(snake_to_pascal("transfer_from"), "TransferFrom");
        assert_eq!(snake_to_pascal("set_name"), "SetName");
        assert_eq!(snake_to_pascal("a"), "A");
        assert_eq!(snake_to_pascal("a_b_c"), "ABC");
        assert_eq!(snake_to_pascal(""), "");
    }

    #[test]
    fn test_snake_to_pascal_leading_underscores() {
        assert_eq!(snake_to_pascal("_private"), "Private");
        assert_eq!(snake_to_pascal("__double"), "Double");
    }
}
