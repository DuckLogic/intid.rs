//! Implements the [`IntegerId`] derive macro.
//!
//! Generally, you want to use the re-export from the `intid` or `idmap` crates.
//! In the `intid` crate this requires explicitly enabling the `derive` feature.
//! In the `idmap` crate, the derive feature is on by default.

use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};

use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{Data, DataStruct, DeriveInput, Expr, ExprLit, Fields, Lit, Member, Type};

/// Implements `intid::IntegerIdContiguous` for a newtype struct.
///
/// This is automatically derived when deriving `IntegerIdCounter`.
#[proc_macro_derive(IntegerIdContiguous, attributes(intid))]
pub fn integer_id_contiguous(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_contiguous(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn impl_contiguous(ast: &DeriveInput) -> syn::Result<TokenStream> {
    const TRAIT_NAME: &str = "IntegerIdContiguous";
    // No need to parse options (we don't care)
    let name = &ast.ident;
    let data = ensure_only_struct(&ast.data, TRAIT_NAME)?;
    let NewtypeStructInfo {
        field_name,
        field_type,
    } = ensure_newtype_struct(&ast.ident, data, TRAIT_NAME)?;
    let field_type_as_contig = quote_spanned! {
        field_type.span() => <#field_type as intid::IntegerIdContiguous>
    };
    Ok(quote! {
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        impl intid::IntegerIdContiguous for #name {
            const MIN_ID: Self = #name {
                #field_name: #field_type_as_contig::MIN_ID,
            };
            const MAX_ID: Self = #name {
                #field_name: #field_type_as_contig::MAX_ID,
            };
        }
    })
}

/// Implements `intid::IntegerIdCounter` for a newtype struct.
///
/// ```rust
/// use intid::{IntegerId, IntegerIdCounter};
/// #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// #[derive(IntegerIdCounter, IntegerId)]
/// struct Example(u32);
/// fn print_counter<T: IntegerIdCounter>() -> String {
///     format!("Starting at {:?}", T::START)
/// }
/// assert_eq!(
///     print_counter::<Example>(),
///     "Starting at Example(0)"
/// );
/// ```
///
/// This will automatically derive `IntegerIdContiguous` trait as well,
/// since that trait is necessary to implement `IntegerIdCounter`.
/// Skip deriving the contiguous trait by using the attribute `#[intid(counter(skip_contiguous))]`:
/// ```rust
/// use intid::{IntegerIdCounter, IntegerId};
/// #[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
/// #[derive(IntegerId, IntegerIdCounter)]
/// #[intid(counter(skip_contiguous))]
/// struct Explicit(u32);
/// impl intid::IntegerIdContiguous for Explicit {
///     const MIN_ID: Self = Explicit(0);
///     const MAX_ID: Self = Explicit(u32::MAX);
/// }
/// ```
#[proc_macro_derive(IntegerIdCounter, attributes(intid))]
pub fn integer_id_counter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_id_counter(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn impl_id_counter(ast: &DeriveInput) -> syn::Result<TokenStream> {
    const TRAIT_NAME: &str = "IntegerIdCounter";
    let options = parse_options(ast)?;
    // No need to parse options (we don't care)
    let name = &ast.ident;
    let data = ensure_only_struct(&ast.data, TRAIT_NAME)?;
    let NewtypeStructInfo {
        field_name,
        field_type,
    } = ensure_newtype_struct(&ast.ident, data, TRAIT_NAME)?;
    let field_type_as_counter = quote_spanned! {
        field_type.span() => <#field_type as intid::IntegerIdCounter>
    };
    let contig_impl = match options.counter {
        Some(ref x) if x.skip_contiguous.is_some() => quote!(),
        None | Some(_) => impl_contiguous(ast)?,
    };
    Ok(quote! {
        #contig_impl
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        impl intid::IntegerIdCounter for #name {
            const START: Self = #name {
                #field_name: #field_type_as_counter::START_INT,
            };
            const START_INT: Self::Int = #field_type_as_counter::START_INT;
        }
    })
}

/// Implements `intid::IntegerId` for a newtype struct or C-like enum.
#[proc_macro_derive(IntegerId, attributes(intid))]
pub fn integer_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_integer_id(&ast)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

// The compiler doesn't seem to know when variables are used in the macro
fn impl_integer_id(ast: &DeriveInput) -> syn::Result<TokenStream> {
    let options = parse_options(ast)?;
    let name = &ast.ident;
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
                    Ok(quote! {
                        #[automatically_derived]
                        #[allow(clippy::init_numbered_fields)]
                        #impl_decl {
                            type Int = #int_type;

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
                    #from_impl
                })
            }
        }
        Data::Union(ref data) => Err(syn::Error::new_spanned(
            data.union_token,
            "Unions are unsupported",
        )),
    }
}

fn ensure_only_struct<'a>(ast: &'a Data, trait_name: &str) -> syn::Result<&'a DataStruct> {
    match ast {
        Data::Struct(ref data) => Ok(data),
        Data::Enum(ref data) => Err(syn::Error::new_spanned(
            data.enum_token,
            format!("Deriving {trait_name} is not currently supported for enums"),
        )),
        Data::Union(ref data) => Err(syn::Error::new_spanned(
            data.union_token,
            format!("Deriving {trait_name} is not supported for unions"),
        )),
    }
}

struct NewtypeStructInfo<'a> {
    field_name: Member,
    field_type: &'a Type,
}

fn ensure_newtype_struct<'a>(
    ident: &Ident,
    data: &'a DataStruct,
    trait_name: &str,
) -> syn::Result<NewtypeStructInfo<'a>> {
    let fields = &data.fields;
    match fields.len() {
        1 => {
            let field = fields.iter().next().unwrap();
            let field_name = field
                .ident
                .clone()
                .map_or_else(|| Member::from(0), Member::from);
            let field_type = &field.ty;
            Ok(NewtypeStructInfo {
                field_name,
                field_type,
            })
        }
        0 => Err(syn::Error::new_spanned(
            ident,
            format!("{trait_name} does not currently support empty structs"),
        )),
        _ => Err(syn::Error::new_spanned(
            fields.iter().nth(1).unwrap(),
            format!("{trait_name} can only be applied to structs with a single field"),
        )),
    }
}

fn parse_options(ast: &DeriveInput) -> syn::Result<MainOptions> {
    ast.attrs
        .iter()
        .find(|attr| attr.meta.path().is_ident("intid"))
        .map(MainOptions::parse_attr)
        .unwrap_or_else(|| Ok(MainOptions::default()))
}

#[derive(Default, Debug)]
struct MainOptions {
    /// Automatically generate a `From<&Self>` implementation
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
