//! Implements the [`IntegerId`] derive macro.
//!
//! Generally, you want to use the re-export from the `intid` or `idmap` crates.
//! In the `intid` crate this requires explicitly enabling the `derive` feature.
//! In the `idmap` crate, the derive feature is on by default.

use proc_macro2::Ident;
use quote::{quote, quote_spanned};

use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Expr, ExprLit, Fields, Lit, Member};

/// Implements [`IntegerId`] for a newetype struct or C-like enum.
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
                    let field_name = field
                        .ident
                        .clone()
                        .map_or_else(|| Member::from(0), Member::from);
                    let field_type = &field.ty;
                    let field_type_as_id = quote_spanned! {
                        field_type.span() => <#field_type as intid::IntegerId>
                    };
                    let int_type = quote_spanned! {
                        field_type.span() => <#field_type as intid::IntegerId>::Int
                    };
                    let int_constructor = |method_name: &str, needs_try: bool| {
                        let maybe_try = if needs_try { quote!(?) } else { quote!() };
                        let method_name = Ident::new(method_name, field.ty.span());
                        quote_spanned! {
                            field_type.span() => #name {
                                #field_name: #field_type_as_id::#method_name(int)#maybe_try
                            }
                        }
                    };
                    let impl_from_int = int_constructor("from_int", false);
                    let impl_from_int_checked = int_constructor("from_int_checked", true);
                    let impl_from_int_unchecked = int_constructor("from_int_unchecked", false);
                    let impl_to_int = quote_spanned! { field_type.span() => #field_type_as_id::to_int(self.#field_name) };
                    let impl_decl =
                        quote_spanned! { name.span() => impl intid::IntegerId for #name };
                    Ok(quote! {
                        #[automatically_derived]
                        #impl_decl {
                            type Int = #int_type;
                            const START: Option<Self> = {
                                // const_if_match stable since 1.46
                                match #field_type_as_id::START {
                                    Some(inner_start) => Some(#name {
                                        #field_name: inner_start,
                                    }),
                                    None => None,
                                }
                            };
                            #[inline]
                            fn from_int(int: #int_type) -> Self {
                                #impl_from_int
                            }
                            #[inline]
                            fn from_int_checked(int: #int_type) -> Option<Self> {
                                Some(#impl_from_int_checked)
                            }
                            #[inline]
                            #[allow(unsafe_code)]
                            unsafe fn from_int_unchecked(int: #int_type) -> Self {
                                // SAFETY: Simply delegating responsibility
                                unsafe { #impl_from_int_unchecked }
                            }
                            #[inline]
                            fn to_int(self) -> #int_type {
                                #impl_to_int
                            }
                        }
                        impl From<&'_ #name> for #name {
                            #[inline]
                            fn from(this: &'_ #name) -> #name {
                                *this
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
                    )) => match value.base10_parse::<usize>() {
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
                // TODO: Dont assume that the repr fits in an usize
                Ok(quote! {
                    impl intid::IntegerId for #name {
                        type Int = usize;

                        // no reasonable start for an enum
                        const START: Option<Self> = None;

                        #[inline]
                        fn from_int_checked(x: usize) -> Option<Self> {
                            Some(match x {
                                #(#variant_matches,)*
                                _ => return None,
                            })
                        }

                        #[inline]
                        unsafe fn from_int_unchecked(x: usize) -> Self {
                            match x {
                                #(#variant_matches,)*
                                _ => {
                                    // SAFETY: Validity guaranteed by caller
                                    unsafe { core::hint::unreachable_unchecked() }
                                }
                            }
                        }


                        #[inline]
                        fn to_int(self) -> usize {
                            self as usize
                        }
                    }
                    impl From<&'_ #name> for #name {
                        #[inline]
                        fn from(this: &'_ #name) -> #name {
                            *this
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
