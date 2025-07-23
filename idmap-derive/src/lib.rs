use quote::quote;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Lit};

#[proc_macro_derive(IntegerId)]
pub fn integer_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_integer_id(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

// The compiler doesn't seem to know when variables are used in the macro
fn impl_integer_id(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &ast.ident;
    match ast.data {
        Data::Struct(ref data) => {
            let fields = &data.fields;
            match fields.len() {
                1 => {
                    let field = fields.iter().next().unwrap();
                    /*
                     * NOTE: Delegating to the field's implementation allows efficient polymorphic overflow handling for all supported types.
                     * New types can be added to the library transparently, without changing the automatically derived implementation.
                     * Existing types can be improved by changing the implementation in one place, without touching the derived implementation.
                     * This should have zero overhead when inlining is enabled, since they're marked inline(always).
                     */
                    let field_type = &field.ty;
                    let (constructor, field_name) = match data.fields {
                        Fields::Named(_) => {
                            let field_name = field.ident.to_token_stream();
                            (quote!(#name { #field_name: value }), field_name)
                        }
                        Fields::Unnamed(_) => (quote! { #name( value ) }, quote!(0)),
                        Fields::Unit => unreachable!(),
                    };
                    Ok(quote! {
                        impl ::idmap::IntegerId for #name {
                            #[inline(always)]
                            fn from_id(id: u64) -> Self {
                                let value = <#field_type as ::idmap::IntegerId>::from_id(id);
                                #constructor
                            }
                            #[inline(always)]
                            fn id(&self) -> u64 {
                                <#field_type as ::idmap::IntegerId>::id(&self.#field_name)
                            }
                            #[inline(always)]
                            fn id32(&self) -> u32 {
                                <#field_type as ::idmap::IntegerId>::id32(&self.#field_name)
                            }
                        }
                    })
                }
                0 => Err(syn::Error::new_spanned(
                    &ast.ident,
                    "IntegerId does not currently support empty structs",
                )),
                _ => Err(syn::Error::new_spanned(
                    fields.iter().nth(1).unwrap(),
                    "IntegerId can only be applied to structs with a single field",
                )),
            }
        }
        Data::Enum(ref data) => {
            let mut idx = 0;
            let mut variant_matches = Vec::new();
            let mut errors = Vec::new();
            for variant in &data.variants {
                let ident = &variant.ident;
                match variant.fields {
                    Fields::Unit => (),
                    _ => errors.push(syn::Error::new_spanned(
                        &variant.fields,
                        "IntegerId can only be applied to C-like enums",
                    )),
                }
                match &variant.discriminant {
                    Some((
                        _,
                        Expr::Lit(ExprLit {
                            lit: Lit::Int(value),
                            ..
                        }),
                    )) => match value.base10_parse::<u64>() {
                        Ok(discriminant) => {
                            idx = discriminant;
                        }
                        Err(x) => errors.push(x),
                    },
                    Some((_, discriminant_expr)) => errors.push(syn::Error::new_spanned(
                        discriminant_expr,
                        "Discriminant too complex to understand",
                    )),
                    None => {}
                }
                variant_matches.push(quote!(#idx => #name::#ident));
                idx += 1;
            }
            let mut errors = errors.into_iter();
            if let Some(mut error) = errors.next() {
                for other in errors {
                    error.combine(other);
                }
                Err(error)
            } else {
                Ok(quote! {
                    impl ::idmap::IntegerId for #name {
                        #[inline]
                        #[track_caller]
                        fn from_id(id: u64) -> Self {
                            match id {
                                #(#variant_matches,)*
                                _ => ::idmap::_invalid_id(id)
                            }
                        }
                        #[inline]
                        fn id(&self) -> u64 {
                            *self as u64
                        }
                        #[inline]
                        fn id32(&self) -> u32 {
                            *self as u32
                        }
                    }
                })
            }
        }
        Data::Union(ref data) => Err(syn::Error::new_spanned(
            data.union_token,
            "Unions are unsupported",
        )),
    }
}
