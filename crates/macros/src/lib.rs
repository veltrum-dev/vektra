//! Vektra 过程宏。

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, Fields, LitStr, Result, Variant, parse_macro_input,
};

/// 为无字段 enum 自动实现 `vektra::IntoIconSource`。
///
/// 每个变体默认映射到 `icons/<snake_case_variant>.svg`。可以使用
/// `#[icon(path = "...")]` 覆盖单个变体的路径。
#[proc_macro_derive(IntoIconSource, attributes(icon))]
pub fn derive_into_icon_source(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_into_icon_source(&input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_into_icon_source(input: &DeriveInput) -> Result<TokenStream2> {
    reject_icon_attributes(&input.attrs)?;

    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "IntoIconSource 只能派生在 enum 上",
        ));
    };

    let vektra = vektra_path();
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let arms = data
        .variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let path = icon_path(variant)?;
            Ok(quote! {
                Self::#ident => #vektra::IconSource::asset(#path),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        impl #impl_generics #vektra::IntoIconSource for #name #ty_generics #where_clause {
            fn into_icon_source(self) -> #vektra::IconSource {
                match self {
                    #(#arms)*
                }
            }
        }
    })
}

fn vektra_path() -> TokenStream2 {
    match crate_name("vektra") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = proc_macro2::Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Err(_) => quote!(::vektra),
    }
}

fn icon_path(variant: &Variant) -> Result<String> {
    if !matches!(variant.fields, Fields::Unit) {
        return Err(Error::new_spanned(
            &variant.fields,
            "IntoIconSource 只支持无字段 enum 变体",
        ));
    }

    let mut explicit_path = None;
    let mut icon_attr_seen = false;

    for attr in variant
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("icon"))
    {
        if icon_attr_seen {
            return Err(Error::new_spanned(
                attr,
                "每个变体只能声明一个 #[icon(...)] 属性",
            ));
        }
        icon_attr_seen = true;

        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("path") {
                return Err(meta.error("只支持 #[icon(path = \"...\")]"));
            }
            if explicit_path.is_some() {
                return Err(meta.error("path 属性不能重复"));
            }

            let value = meta.value()?;
            let literal: LitStr = value.parse()?;
            let path = literal.value();
            if path.is_empty() {
                return Err(Error::new(literal.span(), "图标路径不能为空"));
            }
            explicit_path = Some(path);
            Ok(())
        })?;

        if explicit_path.is_none() {
            return Err(Error::new_spanned(
                attr,
                "#[icon(...)] 必须包含 path = \"...\"",
            ));
        }
    }

    Ok(explicit_path
        .unwrap_or_else(|| format!("icons/{}.svg", variant.ident.to_string().to_snake_case())))
}

fn reject_icon_attributes(attrs: &[Attribute]) -> Result<()> {
    if let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("icon")) {
        return Err(Error::new_spanned(
            attr,
            "#[icon(...)] 只能写在 enum 变体上",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;
    use syn::parse_quote;

    #[test]
    fn default_variant_path_uses_snake_case() {
        let input: DeriveInput = parse_quote! {
            enum AppIconName {
                LogoMark,
            }
        };
        let tokens = expand_into_icon_source(&input)
            .unwrap()
            .into_token_stream()
            .to_string();
        assert!(tokens.contains("\"icons/logo_mark.svg\""));
    }

    #[test]
    fn explicit_path_overrides_default() {
        let input: DeriveInput = parse_quote! {
            enum AppIconName {
                #[icon(path = "icons/favorite_filled.svg")]
                FavoriteFilled,
            }
        };
        let tokens = expand_into_icon_source(&input)
            .unwrap()
            .into_token_stream()
            .to_string();
        assert!(tokens.contains("\"icons/favorite_filled.svg\""));
    }

    #[test]
    fn rejects_struct_input() {
        let input: DeriveInput = parse_quote! {
            struct AppIconName;
        };
        let error = expand_into_icon_source(&input).unwrap_err().to_string();
        assert!(error.contains("只能派生在 enum"));
    }

    #[test]
    fn rejects_union_input() {
        let input: DeriveInput = parse_quote! {
            union AppIconName {
                bits: u8,
            }
        };
        let error = expand_into_icon_source(&input).unwrap_err().to_string();
        assert!(error.contains("只能派生在 enum"));
    }

    #[test]
    fn rejects_variant_fields() {
        let input: DeriveInput = parse_quote! {
            enum AppIconName {
                Logo(String),
            }
        };
        let error = expand_into_icon_source(&input).unwrap_err().to_string();
        assert!(error.contains("无字段 enum 变体"));
    }

    #[test]
    fn rejects_duplicate_path_attributes() {
        let input: DeriveInput = parse_quote! {
            enum AppIconName {
                #[icon(path = "icons/a.svg", path = "icons/b.svg")]
                Logo,
            }
        };
        let error = expand_into_icon_source(&input).unwrap_err().to_string();
        assert!(error.contains("不能重复"));
    }
}
