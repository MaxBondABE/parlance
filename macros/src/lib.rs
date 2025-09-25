use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, LitStr, Meta, MetaNameValue,
    parse_macro_input,
};

#[proc_macro_derive(Parser, attributes(tag, case_sensitive, case_insensitive))]
pub fn derive_parser(input: TokenStream) -> TokenStream {
    // TODO handle single-variant enums
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(enum_name, "#[derive(Parser)] can only be used on enums")
            .to_compile_error()
            .into();
    };
    let default_sensitivity =
        match CaseSensitivity::new(&input.attrs).map(|x| x.unwrap_or_default()) {
            Ok(s) => s,
            Err(e) => return e.to_compile_error().into(),
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

        let tag = match tag_literal(&variant.attrs, variant_name) {
            Ok(tag) => tag,
            Err(e) => return e.to_compile_error().into(),
        };
        let sensitivity =
            match CaseSensitivity::new(&variant.attrs).map(|x| x.unwrap_or(default_sensitivity)) {
                Ok(s) => s,
                Err(e) => return e.to_compile_error().into(),
            };

        let parser = match sensitivity {
            CaseSensitivity::Auto => quote! { ::parlance::primitives::tag::tag_auto(#tag) },
            CaseSensitivity::Sensitive => quote! { ::parlance::primitives::tag::tag(#tag) },
            CaseSensitivity::Insensitive => {
                quote! { ::parlance::primitives::tag::tag_no_case(#tag) }
            }
        };

        quote! {
            ::parlance::parse::Parser::with_output(
                #parser,
                Self::#variant_name
            )
        }
    });

    let parser_impl = quote! {
        const _: () = {
            fn assert_impl_clone<T: Clone>() {}
            let _ = assert_impl_clone::<#enum_name>;
        };

        impl #enum_name {
            pub fn parse<I: ::parlance::input::Input>(input: &I) -> ::parlance::parse::ParserResult<I, Self> {
                (#(#variant_arms,)*).or().parse(input)
            }
        }
    };

    parser_impl.into()
}

fn tag_literal(attrs: &[Attribute], name: &Ident) -> Result<LitStr, syn::Error> {
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
                    _ => {
                        return Err(syn::Error::new_spanned(
                            value,
                            "Tags must be string literals",
                        ));
                    }
                },
                _ => {
                    return Err(syn::Error::new_spanned(
                        value,
                        "Tags must be string literals",
                    ));
                }
            }
        }
    }

    Ok(LitStr::new(
        &tag.unwrap_or(name.to_string().to_lowercase()),
        name.span(),
    ))
}

#[derive(Clone, Copy, Default)]
enum CaseSensitivity {
    #[default]
    Auto,
    Sensitive,
    Insensitive,
}
impl CaseSensitivity {
    pub fn new(attrs: &[Attribute]) -> Result<Option<Self>, syn::Error> {
        let mut output = None;
        for a in attrs.iter() {
            let path = a.path();
            if path.is_ident("case_sensitive") {
                if output.is_some() {
                    return Err(syn::Error::new_spanned(
                        path,
                        "#[case_sensitive] and #[case_insensitive] are mutually exclusive",
                    ));
                }
                output = Some(Self::Sensitive);
            } else if path.is_ident("case_insensitive") {
                if output.is_some() {
                    return Err(syn::Error::new_spanned(
                        path,
                        "#[case_sensitive] and #[case_insensitive] are mutually exclusive",
                    ));
                }
                output = Some(Self::Insensitive);
            }
        }

        Ok(output)
    }
}

#[proc_macro_derive(FromNever)]
pub fn derive_from_never(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let from_impl = quote! {
        impl #impl_generics ::core::convert::From<::parlance::parse::Never>
        for #name #ty_generics #where_clause {
            fn from(_: ::parlance::parse::Never) -> Self {
                unreachable!("Never is never constructed.")
            }
        }
    };

    from_impl.into()
}
