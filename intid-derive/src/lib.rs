//! Implements the [`IntegerId`] derive macro.
//!
//! Generally, you want to use the re-export from the `intid` or `idmap` crates.
//! In the `intid` crate this requires explicitly enabling the `derive` feature.
//! In the `idmap` crate, the derive feature is on by default.
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::DeriveInput;

use crate::analyze::{analyze, AnalyzedType, TargetTrait};

mod analyze;

#[allow(clippy::needless_pass_by_value)]
fn maybe_expand(input: TokenStream, name: &str) -> TokenStream {
    let _ = name;
    #[cfg(not(feature = "expander"))]
    {
        #[cfg(intid_derive_use_expander)]
        {
            compile_error!(
                "Enabled `cfg(intid_derive_use_expander)`, but missing 'expander' feature"
            )
        }
        input
    }
    #[cfg(feature = "expander")]
    {
        let random: u64 = {
            use core::hash::{BuildHasher, Hasher};
            use std::hash::RandomState;
            RandomState::new().build_hasher().finish()
        };
        let input = &input;
        let output = quote! {
            #[allow(clippy::undocumented_unsafe_blocks)]
            const _: () = {
                #input
            };
        };
        // fixes a bug with unit tests conflicting with integration tests
        let expanded = expander::Expander::new(format!("{name}-{random:X}"))
            .fmt(expander::Edition::_2021)
            .verbose(true)
            // It would be nice to use an environment variable here,
            // but that would require `proc_macro::tracked_env`
            .dry(cfg!(not(intid_derive_use_expander)))
            .write_to_out_dir(output)
            .unwrap_or_else(|e| {
                eprintln!("Failed to write to file: {e:?}");
                input.clone()
            });
        expanded
    }
}

/// See `intid` crate for docs.
#[proc_macro_derive(IntegerIdContiguous, attributes(intid))]
pub fn integer_id_contiguous(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    maybe_expand(
        impl_contiguous(&ast).unwrap_or_else(syn::Error::into_compile_error),
        "IntegerIdContiguous",
    )
    .into()
}

fn impl_contiguous(ast: &DeriveInput) -> syn::Result<TokenStream> {
    const TARGET_TRAIT: TargetTrait = TargetTrait::IntegerIdContiguous;
    let analyzed = analyze(ast, TARGET_TRAIT)?;
    impl_contiguous_for(&analyzed)
}

fn impl_contiguous_for(analyzed: &AnalyzedType) -> syn::Result<TokenStream> {
    // No need to parse options (we don't care)
    let newtype = analyzed.ensure_only_newtype()?;
    let name = newtype.ident();
    let wrapped_type = newtype.wrapped_field_type;
    let require_contig = quote_spanned!(newtype.wrapped_field_type.span() => {
        fn require_contig<T: intid::IntegerIdContiguous>() {}
        let _ = require_contig::<#wrapped_type>;
    });
    Ok(quote! {
        const _: () = {
            #require_contig
        };
        #[automatically_derived]
        impl intid::IntegerIdContiguous for #name {}
    })
}

/// See `intid` crate for docs.
#[proc_macro_derive(IntegerIdCounter, attributes(intid))]
pub fn integer_id_counter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    maybe_expand(
        impl_id_counter(&ast).unwrap_or_else(syn::Error::into_compile_error),
        "IntegerIdCounter",
    )
    .into()
}

fn impl_id_counter(ast: &DeriveInput) -> syn::Result<TokenStream> {
    const TARGET_TRAIT: TargetTrait = TargetTrait::IntegerIdCounter;
    let options = parse_options(ast)?;
    // No need to parse options (we don't care)
    let name = &ast.ident;
    let analyzed = analyze(ast, TARGET_TRAIT)?;
    let newtype = analyzed.ensure_only_newtype()?;
    let field_type_as_counter = newtype.wrapped_as(quote!(intid::IntegerIdCounter));
    let contig_impl = match options.counter {
        Some(ref x) if x.skip_contiguous.is_some() => quote!(),
        None | Some(_) => impl_contiguous_for(&analyzed)?,
    };
    let start_int = quote!(#field_type_as_counter::START_INT);
    let start = newtype.construct(&start_int);
    Ok(quote! {
        #contig_impl
        #[automatically_derived]
        impl intid::IntegerIdCounter for #name {
            const START: Self = #start;
            const START_INT: Self::Int = #start_int;
        }
    })
}

/// See the documentation in the `intid` crate for details.
#[proc_macro_derive(IntegerId, attributes(intid))]
pub fn integer_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    maybe_expand(
        impl_integer_id(&ast).unwrap_or_else(syn::Error::into_compile_error),
        "IntegerId",
    )
    .into()
}

// The compiler doesn't seem to know when variables are used in the macro
fn impl_integer_id(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let options = parse_options(ast)?;
    let name = &ast.ident;
    // TODO: Replace From<&'_ #name> with From<#wrapped_type>?
    let from_impl = if options.from.is_none() {
        quote!()
    } else {
        quote! {
            impl From<&'_ #name> for #name {
                #[inline]
                fn from(this: &'_ #name) -> #name {
                    *this
                }
            }
        }
    };
    const TARGET_TRAIT: TargetTrait = TargetTrait::IntegerId;
    let analyzed = analyze::analyze(ast, TARGET_TRAIT)?;
    match analyzed {
        AnalyzedType::NewType(ref tp) => {
            let field_type = tp.wrapped_field_type;
            let field_name = &tp.wrapped_field_name;
            let field_type_as_id = tp.wrapped_as(quote!(intid::IntegerId));
            let int_type = quote_spanned! {
                field_type.span() => #field_type_as_id::Int
            };
            let int_constructor = |method_name: &str, needs_try: bool| {
                let maybe_try = if needs_try { quote!(?) } else { quote!() };
                let method_name = Ident::new(method_name, tp.wrapped_field_type.span());
                tp.construct(quote!(#field_type_as_id::#method_name(int)#maybe_try))
            };
            let impl_from_int = int_constructor("from_int", false);
            let impl_from_int_checked = int_constructor("from_int_checked", true);
            let impl_from_int_unchecked = int_constructor("from_int_unchecked", false);
            let impl_to_int =
                quote_spanned! { field_type.span() => #field_type_as_id::to_int(self.#field_name) };
            let impl_decl = quote_spanned! { name.span() => impl intid::IntegerId for #name };
            let verify_counter_impl = match options.counter {
                Some(CounterOptions { name_span, .. }) => {
                    // If the counter option is used, we should be a counter
                    quote_spanned! { name_span =>
                        {
                            #[inline(always)]
                            fn verify_counter<T: intid::IntegerIdCounter>() {}
                            verify_counter::<#name>();
                        }
                    }
                }
                None => quote!(),
            };
            let field_name = &tp.wrapped_field_name;
            Ok(quote! {
                #[automatically_derived]
                #[allow(clippy::init_numbered_fields)]
                #impl_decl {
                    type Int = #int_type;
                    const MIN_ID: Option<Self> = match #field_type_as_id::MIN_ID {
                        Some(min) => Some(#name { #field_name: min }),
                        None => None,
                    };
                    const MAX_ID: Option<Self> = match #field_type_as_id::MAX_ID {
                        Some(max) => Some(#name { #field_name: max }),
                        None => None,
                    };
                    const MIN_ID_INT: Option<Self::Int> = #field_type_as_id::MIN_ID_INT;
                    const MAX_ID_INT: Option<Self::Int> = #field_type_as_id::MIN_ID_INT;
                    const TRUSTED_RANGE: Option<intid::trusted::TrustedRangeToken<Self>> = {
                        // SAFETY: We simply delegate, so are valid if #field_type is
                        unsafe { intid::trusted::TrustedRangeToken::assume_valid_if::<#field_type>() }
                    };

                    #[inline]
                    fn from_int(int: #int_type) -> Self {
                        #verify_counter_impl
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
                #from_impl
            })
        }
        AnalyzedType::Enum(ref tp) => {
            let variant_matches = tp
                .variants
                .iter()
                .map(|variant| {
                    let idx = variant.discriminant;
                    let variant_name = variant.name();
                    quote!(#idx => #name::#variant_name)
                })
                .collect::<Vec<_>>();
            let int_type = tp.discriminant_type;
            let select_method = |cmp: TokenStream| {
                quote! {
                    const fn select(
                        a: #name,
                        b: #name,
                    ) -> #name {
                        if (a as #int_type) #cmp (b as #int_type) {
                            a
                        } else if (b as #int_type) #cmp (a as #int_type) {
                            b
                        } else {
                            panic!("internal error: detected conflicting enum ids")
                        }
                    }
                }
            };
            let do_select = tp
                .variants
                .iter()
                .map(|x| {
                    let variant_name = x.name();
                    quote!(#name::#variant_name)
                })
                .reduce(|a, b| quote!(select(#a, #b)));
            let select_max = select_method(quote!(>));
            let select_min = select_method(quote!(<));
            let [min_id, max_id] = if tp.is_inhabited() {
                let do_select = do_select.unwrap();
                [select_min, select_max].map(|select_impl| {
                    quote!(Some({
                        #select_impl
                        #do_select
                    }))
                })
            } else {
                [quote!(None), quote!(None)]
            };
            Ok(quote! {
                impl intid::IntegerId for #name {
                    type Int = #int_type;
                    const MAX_ID: Option<Self> = #max_id;
                    const MIN_ID: Option<Self> = #min_id;
                    const MAX_ID_INT: Option<#int_type> = match Self::MAX_ID {
                        Some(max) => Some(max as #int_type),
                        None => None,
                    };
                    const MIN_ID_INT: Option<#int_type> = match Self::MIN_ID {
                        Some(min) => Some(min as #int_type),
                        None => None,
                    };
                    const TRUSTED_RANGE: Option<intid::trusted::TrustedRangeToken<Self>> = {
                        // SAFETY: We accurately report the range of enum discriminants
                        Some(unsafe { intid::trusted::TrustedRangeToken::assume_valid() })
                    };

                    #[inline]
                    #[allow(unreachable_code)]
                    fn from_int_checked(x: #int_type) -> Option<Self> {
                        // NOTE: Works assuming that x fits in u64
                        // Needed since the literals in variant_matches have to have a concrete type
                        const _: () = {
                            assert!(#int_type::BITS <= u64::BITS, "too many bits for derive");
                        };
                        Some(match u64::from(x) {
                            #(#variant_matches,)*
                            _ => return None,
                        })
                    }

                    #[inline]
                    #[allow(unreachable_code)]
                    unsafe fn from_int_unchecked(x: #int_type) -> Self {
                        match u64::from(x) {
                            #(#variant_matches,)*
                            _ => {
                                // SAFETY: Validity guaranteed by caller
                                unsafe { core::hint::unreachable_unchecked() }
                            }
                        }
                    }

                    #[inline]
                    fn to_int(self) -> #int_type {
                        self as #int_type
                    }
                }
                #from_impl
            })
        }
    }
}

fn parse_options(ast: &DeriveInput) -> syn::Result<MainOptions> {
    ast.attrs
        .iter()
        .find(|attr| attr.meta.path().is_ident("intid"))
        .map_or_else(|| Ok(MainOptions::default()), MainOptions::parse_attr)
}

#[derive(Default, Debug)]
struct MainOptions {
    /// Automatically generate a `From<&Self>` implementation.
    ///
    /// TODO: This should instead generate a `From<Inner>` implementation.
    from: Option<Span>,
    /// Options specific to a counter.
    counter: Option<CounterOptions>,
}
impl MainOptions {
    fn parse_attr(attr: &syn::Attribute) -> syn::Result<Self> {
        let mut res = MainOptions::default();
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                res.from = Some(meta.path.span());
                Ok(())
            } else if meta.path.is_ident("counter") {
                if res.counter.is_some() {
                    return Err(syn::Error::new_spanned(
                        &meta.path,
                        "Specified counter twice",
                    ));
                }
                let mut counter_opts = CounterOptions {
                    name_span: meta.path.span(),
                    skip_contiguous: None,
                };
                if meta.input.peek(syn::token::Paren) {
                    meta.parse_nested_meta(|meta| {
                        if meta.path.is_ident("skip_contiguous") {
                            counter_opts.skip_contiguous = Some(meta.path.span());
                            Ok(())
                        } else {
                            Err(meta.error("Invalid `counter` attribute"))
                        }
                    })?;
                }
                res.counter = Some(counter_opts);
                Ok(())
            } else {
                Err(meta.error("Invalid attribute"))
            }
        })?;
        Ok(res)
    }
}

#[derive(Debug)]
struct CounterOptions {
    /// The span for the `counter` ident.
    name_span: Span,
    skip_contiguous: Option<Span>,
}
