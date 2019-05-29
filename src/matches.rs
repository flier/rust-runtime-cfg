//! Evaluation of configuration flags, at runtime-time.

cfg_if! {
    if #[cfg(feature = "std")] {
        use std::collections::HashMap;
        use std::borrow::Borrow;
        use std::hash::Hash;
    } else {
        use alloc::vec::Vec;
    }
}

use crate::Predicate;

/// A matcher for string matching.
pub trait Matcher {
    fn matches(&self, value: &str) -> bool;
}

/// A pattern for configuration matching.
pub trait Pattern {
    fn matches(&self, key: &str, value: Option<&str>) -> bool;
}

impl Matcher for &str {
    fn matches(&self, value: &str) -> bool {
        *self == value
    }
}

impl Matcher for &[&str] {
    fn matches(&self, value: &str) -> bool {
        self.iter().any(|s| *s == value)
    }
}

impl Matcher for Vec<&str> {
    fn matches(&self, value: &str) -> bool {
        self.iter().any(|s| *s == value)
    }
}

impl<T> Matcher for Option<T>
where
    T: Matcher,
{
    fn matches(&self, value: &str) -> bool {
        self.as_ref().map_or(false, |m| m.matches(value))
    }
}

impl<K, V> Pattern for [(K, Option<V>)]
where
    K: Matcher,
    V: Matcher,
{
    fn matches(&self, key: &str, value: Option<&str>) -> bool {
        if let Some(value) = value {
            self.iter()
                .any(|(k, v)| k.matches(key) && v.as_ref().map_or(false, |v| v.matches(value)))
        } else {
            self.iter().any(|(k, _)| k.matches(key))
        }
    }
}

impl<K, V> Pattern for Vec<(K, Option<V>)>
where
    K: Matcher,
    V: Matcher,
{
    fn matches(&self, key: &str, value: Option<&str>) -> bool {
        self.as_slice().matches(key, value)
    }
}

#[cfg(feature = "std")]
impl<K, V> Pattern for HashMap<K, V>
where
    K: Eq + Hash + Borrow<str>,
    V: Matcher,
{
    fn matches(&self, key: &str, value: Option<&str>) -> bool {
        if let Some(value) = value {
            match self.get(key) {
                Some(v) => v.matches(value),
                _ => false,
            }
        } else {
            self.contains_key(key)
        }
    }
}

impl Predicate {
    /// Returns `true` if configuration matches the predicate
    pub fn matches<P: Pattern>(&self, pattern: &P) -> bool {
        use Predicate::*;

        match self {
            Any(predicates) => predicates
                .iter()
                .any(|predicate| predicate.matches(pattern)),
            All(predicates) => predicates
                .iter()
                .all(|predicate| predicate.matches(pattern)),
            Not(predicate) => !predicate.matches(pattern),
            Name(name) => pattern.matches(name, None),
            NameValue(name, value) => pattern.matches(name, Some(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    cfg_if! {
        if #[cfg(feature = "std")] {
            use quote::quote;
        } else {
            use alloc::vec;
            use alloc::borrow::ToOwned;
            use alloc::boxed::Box;
        }
    }

    use crate::{Cfg, Predicate::*};

    #[test]
    fn test_matches() {
        let testcases = vec![
            (Cfg(Name("unix".to_owned())), vec![("unix", None)], true),
            (
                Cfg(NameValue("target_os".to_owned(), "macos".to_owned())),
                vec![("target_os", Some("macos"))],
                true,
            ),
            (
                Cfg(Any(vec![
                    Box::new(Name("foo".to_owned())),
                    Box::new(Name("bar".to_owned())),
                ])),
                vec![("foo", None)],
                true,
            ),
            (
                Cfg(Not(Box::new(Name("bar".to_owned())))),
                vec![("foo", None), ("bar", None)],
                false,
            ),
            (
                Cfg(All(vec![
                    Box::new(Name("unix".to_owned())),
                    Box::new(NameValue(
                        "target_pointer_width".to_owned(),
                        "32".to_owned(),
                    )),
                ])),
                vec![("unix", None)],
                false,
            ),
            (
                Cfg(All(vec![
                    Box::new(Name("unix".to_owned())),
                    Box::new(NameValue(
                        "target_pointer_width".to_owned(),
                        "32".to_owned(),
                    )),
                ])),
                vec![("unix", None), ("target_pointer_width", Some("32"))],
                true,
            ),
        ];

        for (cfg, flags, res) in testcases {
            assert_eq!(cfg.matches(&flags), res);
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_matches_hashmap() {
        use std::collections::HashMap;

        let mut h = HashMap::new();

        h.insert("foo", None);
        h.insert("bar", None);
        h.insert("target_os", Some(vec!["macos"]));
        h.insert("target_pointer_width", Some(vec!["32"]));

        let testcases = vec![
            (quote! { #[cfg(unix)] }, false),
            (quote! { #[cfg(target_os = "macos")] }, true),
            (quote! { #[cfg(any(foo, bar))] }, true),
            (quote! { #[cfg(not(bar))] }, false),
            (
                quote! { #[cfg(all(unix, target_pointer_width = "32"))] },
                false,
            ),
            (
                quote! { #[cfg(any(unix, target_pointer_width = "32"))] },
                true,
            ),
        ];

        for (s, res) in testcases {
            let err = quote!(#s);

            assert_eq!(
                syn::parse2::<Cfg>(s).unwrap().matches(&h),
                res,
                "matching {}",
                err
            );
        }
    }
}
