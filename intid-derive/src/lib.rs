//! Implements the [`IntegerId`] derive macro.
//!
//! Generally, you want to use the re-export from the `intid` or `idmap` crates.
//! In the `intid` crate this requires explicitly enabling the `derive` feature.
//! In the `idmap` crate, the derive feature is on by default.
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

use core::fmt::{Display, Formatter, Write};
use core::str::FromStr;

use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};

use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{Data, DataStruct, DeriveInput, Expr, ExprLit, Fields, Lit, Member, Type};

#[allow(clippy::needless_pass_by_value)]
fn maybe_expand(input: TokenStream) -> TokenStream {
    #[cfg(not(feature = "_internal-use-expander"))]
    {
        input
    }
    #[cfg(feature = "_internal-use-expander")]
    {
        let input = &input;
        let output = quote! {
            #[allow(clippy::undocumented_unsafe_blocks)]
            const _: () = {
                #input
            };
        };
        let expanded = expander::Expander::new("intid")
            .fmt(expander::Edition::_2021)
            .verbose(true)
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
    maybe_expand(impl_contiguous(&ast).unwrap_or_else(syn::Error::into_compile_error)).into()
}

fn impl_contiguous(ast: &DeriveInput) -> syn::Result<TokenStream> {
    const TRAIT_NAME: &str = "IntegerIdContiguous";
    // No need to parse options (we don't care)
    let name = &ast.ident;
    let data = ensure_only_struct(&ast.data, TRAIT_NAME)?;
    let NewtypeStructInfo {
        field_name: _,
        field_type,
    } = ensure_newtype_struct(&ast.ident, data, TRAIT_NAME)?;
    let require_contig = quote_spanned!(field_type.span() => {
        fn require_contig<T: intid::IntegerIdContiguous>() {}
        let _ = require_contig::<#field_type>;
    });
    Ok(quote! {
        const _: () = {
            #require_contig
        };
        #[automatically_derived]
        #[allow(clippy::init_numbered_fields)]
        impl intid::IntegerIdContiguous for #name {}
    })
}

/// See `intid` crate for docs.
#[proc_macro_derive(IntegerIdCounter, attributes(intid))]
pub fn integer_id_counter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    maybe_expand(impl_id_counter(&ast).unwrap_or_else(syn::Error::into_compile_error)).into()
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

/// See the documentation in the `intid` crate for details.
#[proc_macro_derive(IntegerId, attributes(intid))]
pub fn integer_id(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    maybe_expand(impl_integer_id(&ast).unwrap_or_else(syn::Error::into_compile_error)).into()
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
                            const MIN_ID: Self = #name { #field_name: #field_type_as_id::MIN_ID };
                            const MAX_ID: Self = #name { #field_name: #field_type_as_id::MAX_ID };
                            const MIN_ID_INT: Self::Int = #field_type_as_id::MIN_ID_INT;
                            const MAX_ID_INT: Self::Int = #field_type_as_id::MIN_ID_INT;
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
            let mut idx = 0u64;
            let mut variant_matches = Vec::new();
            let mut errors = Vec::new();
            let repr = determine_repr(ast)?;
            if data.variants.is_empty() {
                return Err(syn::Error::new(Span::call_site(), "Enum must be inhabited"));
            }
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
                idx = idx.checked_add(1).expect("discriminant overflow");
            }
            let int_type = match repr {
                None | Some(Repr::C(_)) => {
                    let ctx = format!("(indexes in [0, {idx}) range)");
                    let needed_bits = idx
                        .checked_next_power_of_two()
                        .unwrap_or_else(|| panic!("Failed to determine discriminant size {ctx}"))
                        .max(8); // everything needs at least 8 bits
                    assert!(needed_bits <= 64, "too many bits for discriminant {ctx}");
                    IntType {
                        bits: Some(u32::try_from(needed_bits).unwrap()),
                        signed: false,
                        span: Span::call_site(),
                    }
                }
                Some(Repr::Integer(value)) => value,
                Some(other) => {
                    return Err(syn::Error::new(
                        other.span(),
                        "failed to determine discrimiant type for {other}",
                    ))
                }
            };
            let mut errors = errors.into_iter();
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
            let do_select = data
                .variants
                .iter()
                .map(|x| quote!(#name::#x))
                .reduce(|a, b| quote!(select(#a, #b)))
                .unwrap();
            let select_max = select_method(quote!(>));
            let select_min = select_method(quote!(<));
            if let Some(mut error) = errors.next() {
                for other in errors {
                    error.combine(other);
                }
                Err(error)
            } else {
                Ok(quote! {
                    impl intid::IntegerId for #name {
                        type Int = #int_type;
                        const MAX_ID: Self = {
                            #select_max
                            #do_select
                        };
                        const MIN_ID: Self = {
                            #select_min
                            #do_select
                        };
                        const MAX_ID_INT: #int_type = Self::MAX_ID as #int_type;
                        const MIN_ID_INT: #int_type = Self::MIN_ID as #int_type;
                        const TRUSTED_RANGE: Option<intid::trusted::TrustedRangeToken<Self>> = {
                            // SAFETY: We accurately report the range of enum discriminants
                            Some(unsafe { intid::trusted::TrustedRangeToken::assume_valid() })
                        };

                        #[inline]
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
        .map_or_else(|| Ok(MainOptions::default()), MainOptions::parse_attr)
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

#[derive(Debug, Copy, Clone)]
struct IntType {
    signed: bool,
    // If `None`, this is a usize/isize
    bits: Option<u32>,
    span: Span,
}
impl quote::ToTokens for IntType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = Ident::new(&self.to_string(), self.span);
        ident.to_tokens(tokens);
    }
}
impl Display for IntType {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        f.write_char(if self.signed { 'i' } else { 'u' })?;
        if let Some(bits) = self.bits {
            write!(f, "{bits}")
        } else {
            f.write_str("size")
        }
    }
}
impl Eq for IntType {}
impl PartialEq for IntType {
    fn eq(&self, other: &Self) -> bool {
        self.signed == other.signed && self.bits == other.bits
    }
}
impl FromStr for IntType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let signed = match s.chars().next() {
            Some('i') => true,
            Some('u') => false,
            _ => return Err(()),
        };
        let s = &s[1..];
        let bits = if s == "size" {
            None
        } else {
            let bits = u32::from_str(s).map_err(|_| ())?;
            match bits {
                8 | 16 | 32 | 64 | 128 => Some(bits),
                _ => return Err(()),
            }
        };
        Ok(IntType {
            bits,
            signed,
            span: Span::call_site(),
        })
    }
}

#[derive(Debug, Copy, Clone)]
enum Repr {
    C(Span),
    Transparent(Span),
    Integer(IntType),
}
impl Repr {
    pub fn span(&self) -> Span {
        match *self {
            Repr::C(span) | Repr::Transparent(span) => span,
            Repr::Integer(ref inner) => inner.span,
        }
    }
}
impl Display for Repr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("#[repr(")?;
        match self {
            Repr::C(_) => f.write_char('C')?,
            Repr::Transparent(_) => f.write_str("transparent")?,
            Repr::Integer(inner) => Display::fmt(&inner, f)?,
        }
        f.write_char('}')
    }
}

fn determine_repr(input: &DeriveInput) -> Result<Option<Repr>, syn::Error> {
    let mut result = None;
    for attr in &input.attrs {
        if attr.meta.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if result.is_some() {
                    return Err(meta.error("Encountered multiple repr(...) attributes"));
                }
                let ident = meta.path.require_ident()?;
                let s = ident.to_string();
                result = Some(match &*s {
                    "C" => Repr::C(ident.span()),
                    "transparent" => Repr::Transparent(ident.span()),
                    x if x.parse::<IntType>().is_ok() => {
                        let int = x.parse::<IntType>().unwrap();
                        Repr::Integer(IntType {
                            span: ident.span(),
                            ..int
                        })
                    }
                    _ => return Err(syn::Error::new(meta.path.span(), "Unknown #[repr])")),
                });
                Ok(())
            })?;
        }
    }
    Ok(result)
}
