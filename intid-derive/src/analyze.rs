use core::fmt::{Display, Formatter, Write};
use core::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::spanned::Spanned;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Expr, ExprLit, Fields, Lit, Member, Type, Variant,
};

macro_rules! define_target_traits {
    ($($target:ident),+ $(,)?) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #[allow(clippy::enum_variant_names)]
        pub enum TargetTrait {
            $($target),*
        }
        impl TargetTrait {
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$target => stringify!($target),)*
                }
            }
            pub fn ident(&self, span: Span) -> Ident {
                Ident::new(self.name(), span)
            }
        }
        impl Display for TargetTrait {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                f.write_str(self.name())
            }
        }
        impl ToTokens for TargetTrait {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                tokens.append(self.ident(Span::call_site()))
            }
        }
    };
}
define_target_traits!(IntegerId, IntegerIdContiguous, IntegerIdCounter, EnumId);

pub fn analyze(
    ast: &DeriveInput,
    target_trait: TargetTrait,
) -> Result<AnalyzedType<'_>, syn::Error> {
    let common = CommonTypeInfo {
        target: target_trait,
        input: ast,
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
                    Ok(AnalyzedType::NewType(AnalyzedNewType {
                        data,
                        wrapped_field_type: field_type,
                        wrapped_field_name: field_name,
                        common,
                    }))
                }
                0 => Err(syn::Error::new_spanned(
                    &ast.ident,
                    format!("{target_trait} does not currently support empty structs",),
                )),
                _ => Err(syn::Error::new_spanned(
                    fields.iter().nth(1).unwrap(),
                    format!("{target_trait} can only be applied to newtype structs"),
                )),
            }
        }
        Data::Enum(ref data) => {
            let mut idx = 0u64;
            let mut analyzed_variants = Vec::new();
            let mut errors = ErrorSet::new();
            let repr = determine_repr(ast)?;
            for variant in &data.variants {
                match variant.fields {
                    Fields::Unit => (),
                    _ => errors.push(syn::Error::new_spanned(
                        &variant.fields,
                        format!("{target_trait} can only be applied to C-like enums"),
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
                analyzed_variants.push(AnalyzedVariant {
                    variant,
                    discriminant: idx,
                });
                idx = idx.checked_add(1).expect("discriminant overflow");
            }
            let discriminant_type = match repr {
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
                        format!("failed to determine discriminant type for {other}",),
                    ))
                }
            };
            errors.finish()?;
            Ok(AnalyzedType::Enum(AnalyzedEnum {
                common,
                discriminant_type,
                variants: analyzed_variants,
                data,
            }))
        }
        Data::Union(ref data) => Err(syn::Error::new_spanned(
            data.union_token,
            format!("Unions are not supported by {target_trait}"),
        )),
    }
}

pub struct CommonTypeInfo<'a> {
    pub target: TargetTrait,
    pub input: &'a DeriveInput,
}
pub enum AnalyzedType<'a> {
    NewType(AnalyzedNewType<'a>),
    Enum(AnalyzedEnum<'a>),
}
impl AnalyzedType<'_> {
    fn common(&self) -> &'_ CommonTypeInfo<'_> {
        match self {
            AnalyzedType::NewType(ref tp) => &tp.common,
            AnalyzedType::Enum(ref tp) => &tp.common,
        }
    }
}
impl AnalyzedType<'_> {
    pub fn ensure_only_newtype(&self) -> syn::Result<&'_ AnalyzedNewType<'_>> {
        let trait_name = self.common().target;
        match self {
            AnalyzedType::NewType(ref tp) => Ok(tp),
            AnalyzedType::Enum(ref tp) => Err(syn::Error::new_spanned(
                tp.data.enum_token,
                format!("Deriving {trait_name} is not currently supported for enums"),
            )),
        }
    }
    pub fn ensure_only_enum(&self) -> syn::Result<&'_ AnalyzedEnum<'_>> {
        let trait_name = self.common().target;
        match self {
            AnalyzedType::Enum(ref tp) => Ok(tp),
            AnalyzedType::NewType(ref tp) => Err(syn::Error::new_spanned(
                tp.data.struct_token,
                format!("Deriving {trait_name} is not currently supported for structs"),
            )),
        }
    }
}
pub struct AnalyzedNewType<'a> {
    pub common: CommonTypeInfo<'a>,
    pub data: &'a DataStruct,
    pub wrapped_field_name: Member,
    pub wrapped_field_type: &'a Type,
}
impl AnalyzedNewType<'_> {
    pub fn ident(&self) -> &'_ Ident {
        &self.common.input.ident
    }
    /// Refer to the wrapped type cast to a specific trait.
    pub fn wrapped_as(&self, target: impl ToTokens) -> TokenStream {
        let wrapped = self.wrapped_field_type;
        quote_spanned!(self.wrapped_field_type.span() => <#wrapped as #target>)
    }
    pub fn construct(&self, value: impl ToTokens) -> TokenStream {
        let value = value.into_token_stream();
        let span = value.span();
        let type_name = self.ident();
        match self.wrapped_field_name {
            Member::Named(ref field_name) => {
                quote_spanned!(span => #type_name { #field_name: #value })
            }
            Member::Unnamed(_) => quote_spanned!(span => #type_name(#value)),
        }
    }
}
pub struct AnalyzedEnum<'a> {
    pub common: CommonTypeInfo<'a>,
    pub data: &'a DataEnum,
    pub variants: Vec<AnalyzedVariant<'a>>,
    pub discriminant_type: IntType,
}
impl AnalyzedEnum<'_> {
    pub fn is_inhabited(&self) -> bool {
        !self.variants.is_empty()
    }
    /// Determine the variants of this enum with the minimum and maximum id.
    ///
    /// This has of type `Self`, not of an integer
    pub fn determine_id_bounds(&self) -> Result<EnumIdBounds, UninhabitedEnumError> {
        let name = &self.common.input.ident;
        let int_type = self.discriminant_type;
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
        let do_select = self
            .variants
            .iter()
            .map(|x| {
                let variant_name = x.name();
                quote!(#name::#variant_name)
            })
            .reduce(|a, b| quote!(select(#a, #b)));
        let select_max = select_method(quote!(>));
        let select_min = select_method(quote!(<));
        if self.is_inhabited() {
            let do_select = do_select.unwrap();
            let [min_id, max_id] = [select_min, select_max].map(|select_impl| {
                quote!({
                    #select_impl
                    #do_select
                })
            });
            Ok(EnumIdBounds { min_id, max_id })
        } else {
            Err(UninhabitedEnumError)
        }
    }
}
#[derive(Clone, Debug)]
pub struct UninhabitedEnumError;
#[derive(Debug)]
pub struct EnumIdBounds<T = TokenStream> {
    pub min_id: T,
    pub max_id: T,
}
impl<T> EnumIdBounds<T> {
    pub fn map<U>(self, mut func: impl FnMut(T) -> U) -> EnumIdBounds<U> {
        EnumIdBounds {
            min_id: func(self.min_id),
            max_id: func(self.max_id),
        }
    }
}
pub struct AnalyzedVariant<'a> {
    pub discriminant: u64,
    pub variant: &'a Variant,
}
impl AnalyzedVariant<'_> {
    #[inline]
    pub fn name(&self) -> &'_ Ident {
        &self.variant.ident
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Repr {
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

pub fn determine_repr(input: &DeriveInput) -> Result<Option<Repr>, syn::Error> {
    let mut result = None;
    for attr in &input.attrs {
        if attr.meta.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                if result.is_some() {
                    return Err(meta.error("Encountered multiple repr(...) attributes"));
                }
                let unknown_repr = || syn::Error::new(meta.path.span(), "Unknown #[repr])");
                let ident = meta.path.get_ident().ok_or_else(unknown_repr)?;
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
                    _ => return Err(unknown_repr()),
                });
                Ok(())
            })?;
        }
    }
    Ok(result)
}

#[must_use = "errors ignored if not used"]
pub struct ErrorSet(pub Vec<syn::Error>);
impl ErrorSet {
    pub const fn new() -> ErrorSet {
        ErrorSet(Vec::new())
    }
    pub fn push(&mut self, e: syn::Error) {
        self.0.push(e);
    }
    pub fn flush(&mut self) -> Result<(), syn::Error> {
        let mut errors = self.0.drain(..);
        if let Some(mut first) = errors.next() {
            for other in errors {
                first.combine(other);
            }
            Err(first)
        } else {
            Ok(())
        }
    }
    pub fn finish(mut self) -> Result<(), syn::Error> {
        self.flush()?;
        assert!(self.0.is_empty());
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IntType {
    signed: bool,
    // If `None`, this is a usize/isize
    bits: Option<u32>,
    span: Span,
}
impl ToTokens for IntType {
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
