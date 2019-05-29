use core::convert::{TryFrom, TryInto};
use core::str::FromStr;

use syn::{bracketed, spanned::Spanned, Token};

use crate::{Cfg, Predicate};

impl FromStr for Cfg {
    type Err = syn::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Cfg::parse(s)
    }
}

impl syn::parse::Parse for Cfg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![#] = input.parse()?;
        let content;
        let _ = bracketed!(content in input);
        let lookahead = content.lookahead1();
        if lookahead.peek(syn::Ident) {
            content.parse::<syn::Meta>().and_then(Cfg::try_from)
        } else {
            Err(lookahead.error())
        }
    }
}

/// A runtime configuration to match flags.
pub fn cfg<T: TryInto<Cfg>>(from: T) -> Result<Cfg, <T as TryInto<Cfg>>::Error> {
    from.try_into()
}

impl TryFrom<proc_macro2::TokenStream> for Cfg {
    type Error = syn::Error;

    fn try_from(s: proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        syn::parse2(s)
    }
}

impl<'ast> TryFrom<&'ast syn::Attribute> for Cfg {
    type Error = syn::Error;

    fn try_from(attr: &'ast syn::Attribute) -> Result<Self, Self::Error> {
        attr.parse_meta().and_then(Cfg::try_from)
    }
}

impl TryFrom<syn::Attribute> for Cfg {
    type Error = syn::Error;

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        attr.parse_meta().and_then(Cfg::try_from)
    }
}

impl TryFrom<syn::Meta> for Cfg {
    type Error = syn::Error;

    fn try_from(meta: syn::Meta) -> Result<Self, Self::Error> {
        if meta.name() == "cfg" {
            parse_meta(meta).map(Cfg)
        } else {
            Err(syn::Error::new(meta.span(), "expect #[cfg(..)] attribute"))
        }
    }
}

impl Cfg {
    /// Find and parse the `cfg` attribute
    pub fn find<'ast>(attrs: impl IntoIterator<Item = &'ast syn::Attribute>) -> Option<Cfg> {
        attrs
            .into_iter()
            .find(|attr| attr.path.is_ident("cfg"))
            .and_then(|attr| Cfg::try_from(attr).ok())
    }

    /// Parse the `cfg` attribute from `meta`
    pub fn parse<S: AsRef<str>>(s: S) -> syn::Result<Self> {
        syn::parse_str(s.as_ref())
    }
}

fn parse_meta(meta: syn::Meta) -> syn::Result<Predicate> {
    match meta {
        syn::Meta::Word(value) => Ok(Predicate::Name(value.to_string())),
        syn::Meta::NameValue(syn::MetaNameValue { ident, lit, .. }) => {
            Ok(Predicate::NameValue(ident.to_string(), lit_to_string(lit)))
        }
        syn::Meta::List(meta_list) => parse_meta_list(meta_list),
    }
}

fn parse_meta_list(meta_list: syn::MetaList) -> syn::Result<Predicate> {
    let span = meta_list.span();
    let syn::MetaList { ident, nested, .. } = meta_list;

    if ident == "any" {
        nested
            .into_iter()
            .map(parse_nested_meta)
            .map(|meta| meta.map(Box::new))
            .collect::<syn::Result<Vec<_>>>()
            .map(Predicate::Any)
    } else if ident == "all" {
        nested
            .into_iter()
            .map(parse_nested_meta)
            .map(|meta| meta.map(Box::new))
            .collect::<syn::Result<Vec<_>>>()
            .map(Predicate::All)
    } else if ident == "not" {
        let mut predicates = nested.into_iter();
        let predicate = predicates
            .next()
            .ok_or_else(|| syn::Error::new(span, "#[cfg(not(..))] predicate can't be empty"))
            .and_then(parse_nested_meta)
            .map(Box::new)
            .map(Predicate::Not);

        if let Some(nested_meta) = predicates.next() {
            Err(syn::Error::new(
                nested_meta.span(),
                "#[cfg(not(..))] only support one predicate",
            ))
        } else {
            predicate
        }
    } else if ident == "cfg" {
        let mut predicates = nested.into_iter();
        let predicate = predicates
            .next()
            .ok_or_else(|| syn::Error::new(span, "#[cfg(..)] predicate can't be empty"))
            .and_then(parse_nested_meta);

        if let Some(nested_meta) = predicates.next() {
            Err(syn::Error::new(
                nested_meta.span(),
                "#[cfg(..)] only support one predicate",
            ))
        } else {
            predicate
        }
    } else {
        Err(syn::Error::new(
            span,
            format!("unexpected operator `{}`", ident),
        ))
    }
}

fn parse_nested_meta(nested_meta: syn::NestedMeta) -> syn::Result<Predicate> {
    let span = nested_meta.span();

    match nested_meta {
        syn::NestedMeta::Meta(meta) => parse_meta(meta),
        syn::NestedMeta::Literal(lit) => Err(syn::Error::new(
            span,
            format!("unexpected literal: {:?}", lit_to_string(lit)),
        )),
    }
}

fn lit_to_string(lit: syn::Lit) -> String {
    use syn::Lit::*;

    match lit {
        Str(v) => v.value(),
        ByteStr(v) => String::from_utf8(v.value()).expect("utf-8"),
        Byte(v) => (v.value() as char).to_string(),
        Char(v) => v.value().to_string(),
        Int(v) => v.value().to_string(),
        Float(v) => v.value().to_string(),
        Bool(v) => v.value.to_string(),
        Verbatim(v) => v.token.to_string(),
    }
}

#[cfg(test)]
mod tests {
    cfg_if! {
        if #[cfg(not(feature = "std"))] {
            use alloc::borrow::ToOwned;
            use alloc::string::ToString;
            use alloc::vec;
        }
    }

    use quote::quote;

    use crate::Predicate::*;

    use super::*;

    #[test]
    fn test_lit() {
        let testcases = vec![
            (quote!("hello world"), "hello world"),
            (quote!(b"hello world"), "hello world"),
            (quote!(b'b'), "b"),
            (quote!('c'), "c"),
            (quote!(123), "123"),
            (quote!(3.14), "3.14"),
            (quote!(true), "true"),
            (quote!(false), "false"),
        ];

        for (s, l) in testcases {
            assert_eq!(
                lit_to_string(syn::parse2::<syn::Lit>(s).unwrap()).as_str(),
                l
            );
        }
    }

    #[test]
    fn test_parse() {
        let testcases = vec![
            (
                quote! { #[cfg(any(foo, bar))] },
                Cfg(Any(vec![
                    Box::new(Name("foo".to_owned())),
                    Box::new(Name("bar".to_owned())),
                ])),
            ),
            (
                quote! { #[cfg(target_os = "macos")] },
                Cfg(NameValue("target_os".to_owned(), "macos".to_owned())),
            ),
            (
                quote! { #[cfg(all(unix, target_pointer_width = "32"))] },
                Cfg(All(vec![
                    Box::new(Name("unix".to_owned())),
                    Box::new(NameValue(
                        "target_pointer_width".to_owned(),
                        "32".to_owned(),
                    )),
                ])),
            ),
            (
                quote! { #[cfg(not(foo))] },
                Cfg(Not(Box::new(Name("foo".to_owned())))),
            ),
            (quote! { #[cfg(test)] }, Cfg(Name("test".to_owned()))),
        ];

        for (ref s, ref cfg) in testcases {
            assert_eq!(syn::parse2::<Cfg>(s.clone()).unwrap(), *cfg, "parse {}", s);
            assert_eq!(
                cfg.to_string().replace(" ", ""),
                s.to_string().replace(" ", "")
            );
        }
    }

    #[test]
    fn test_parse_error() {
        let errcases = vec![
            (quote! { #[test] }, "expect #[cfg(..)] attribute"),
            (quote! { #[cfg(foo(bar))] }, "unexpected operator `foo`"),
            (
                quote! { #[cfg(foo, bar)]},
                "#[cfg(..)] only support one predicate",
            ),
            (
                quote! { #[cfg(not(foo, bar))] },
                "#[cfg(not(..))] only support one predicate",
            ),
            (
                quote! { #[cfg(not())] },
                "#[cfg(not(..))] predicate can't be empty",
            ),
            (quote! { #[cfg()] }, "#[cfg(..)] predicate can't be empty"),
            (quote! { #[cfg("hello")] }, "unexpected literal: \"hello\""),
        ];

        for (s, err) in errcases {
            assert_eq!(syn::parse2::<Cfg>(s).unwrap_err().to_string(), err,);
        }
    }
}
