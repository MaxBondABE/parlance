use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, LitStr, Meta, MetaNameValue,
    parse_macro_input,
};

#[proc_macro_derive(Parser, attributes(tag, case_sensitive, case_insensitive))]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(enum_name, "#[derive(Parser)] can only be used on enums")
            .to_compile_error()
            .into();
    };
    let Ok(default_sensitivity) = CaseSensitivity::new(&input.attrs).map(|x| x.unwrap_or_default())
    else {
        return syn::Error::new_spanned(
            enum_name,
            "Cannot specify both #[case_sensitive] and #[case_insensitive]",
        )
        .to_compile_error()
        .into();
    };

    let variant_arms = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        if !matches!(variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                variant_name,
                "Only unit variants are supported in #[derive(Parser)]",
            )
            .to_compile_error()
            .into();
        }

        let tag = extract_tag(&variant.attrs, variant_name);

        let Ok(sensitivity) =
            CaseSensitivity::new(&variant.attrs).map(|x| x.unwrap_or(default_sensitivity))
        else {
            return syn::Error::new_spanned(
                variant_name,
                "Cannot specify both #[case_sensitive] and #[case_insensitive]",
            )
            .to_compile_error()
            .into();
        };

        let parser_expr = match sensitivity {
            CaseSensitivity::Auto => quote! { ::parlance::primitives::tag::tag_auto(#tag) },
            CaseSensitivity::Sensitive => quote! { ::parlance::primitives::tag::tag(#tag) },
            CaseSensitivity::Insensitive => {
                quote! { ::parlance::primitives::tag::tag_no_case(#tag) }
            }
        };

        quote! {
            #parser_expr.with_output(Self::#variant_name)
        }
    });

    let expanded = quote! {
        impl #enum_name {
            pub fn parse<I: ::parlance::input::Input>(input: &I) -> ::parlance::parse::ParserResult<I, Self> {
                (#(#variant_arms,)*).or().parse(input)
            }
        }
    };

    expanded.into()
}

fn extract_tag(attrs: &[Attribute], name: &Ident) -> LitStr {
    let mut tag = None;
    for attr in attrs {
        if attr.path().is_ident("tag") {
            let Meta::NameValue(MetaNameValue { value, .. }) = &attr.meta else {
                panic!("Tags must be string literals");
            };

            match value {
                Expr::Lit(ExprLit { lit, .. }) => match lit {
                    Lit::Str(s) => {
                        tag = Some(s.value());
                        break;
                    }
                    _ => panic!("Tags must be string literals"),
                },
                _ => panic!("Tags must be string literals"),
            }
        }
    }

    LitStr::new(&tag.unwrap_or(name.to_string().to_lowercase()), name.span())
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CaseSensitivity {
    #[default]
    Auto,
    Sensitive,
    Insensitive,
}
impl CaseSensitivity {
    pub fn new(attrs: &[Attribute]) -> Result<Option<Self>, ()> {
        let mut output = None;
        for a in attrs.iter() {
            let path = a.path();
            if path.is_ident("case_sensitive") {
                if output.is_some() {
                    return Err(());
                }
                output = Some(Self::Sensitive);
            } else if path.is_ident("case_insensitive") {
                if output.is_some() {
                    return Err(());
                }
                output = Some(Self::Insensitive);
            }
        }

        Ok(output)
    }
}
