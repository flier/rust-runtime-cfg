//! Evaluation of configuration flags, at runtime-time.
//!
//! # Example
//!
//! ```
//! use std::convert::TryFrom;
//!
//! use quote::quote;
//!
//! use runtime_cfg::*;
//!
//! # #[cfg(not(feature = "parsing"))] fn main() {}
//! # #[cfg(feature = "parsing")] fn main() {
//! let cfg = quote! { #[cfg(all(unix, target_pointer_width = "32"))] };
//!
//! let cfg = Cfg::try_from(cfg).unwrap();
//! assert_eq!(cfg, all(vec![name("unix"), name_value("target_pointer_width", "32")]).into());
//!
//! let flags = vec![("unix", None), ("target_pointer_width", Some("32"))];
//! assert!(cfg.matches(&flags));
//! # }
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate cfg_if;

pub mod matches;

cfg_if! {
    if #[cfg(feature = "parsing")] {
        mod parsing;

        pub use parsing::cfg;
    }
}

#[cfg(feature = "printing")]
mod printing;

cfg_if! {
    if #[cfg(not(feature = "std"))] {
        extern crate alloc;

        use alloc::boxed::Box;
        use alloc::string::String;
        use alloc::vec::Vec;
    }
}

use core::convert::{AsMut, AsRef};
use core::ops::{Deref, DerefMut};

/// Boolean evaluation of configuration flags, at runtime-time.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Cfg(Predicate);

impl Deref for Cfg {
    type Target = Predicate;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cfg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Predicate> for Cfg {
    fn as_ref(&self) -> &Predicate {
        &self.0
    }
}

impl AsMut<Predicate> for Cfg {
    fn as_mut(&mut self) -> &mut Predicate {
        &mut self.0
    }
}

impl From<Predicate> for Cfg {
    fn from(predicate: Predicate) -> Self {
        Cfg(predicate)
    }
}

impl From<Cfg> for Predicate {
    fn from(cfg: Cfg) -> Self {
        cfg.0
    }
}

/// A configuration predicate.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Predicate {
    /// A configuration predicate success when `any` of sub-predicates success.
    Any(Vec<Box<Predicate>>),
    /// A configuration predicate success when `all` of sub-predicates success.
    All(Vec<Box<Predicate>>),
    /// A configuration predicate apply `not` operator to a predicate.
    Not(Box<Predicate>),
    /// A configuration predicate with name.
    Name(String),
    /// A configuration predicate with name and value.
    NameValue(String, String),
}

/// A configuration predicate success when `any` of sub-predicates success.
pub fn any<I: IntoIterator<Item = Predicate>>(predicates: I) -> Predicate {
    Predicate::Any(predicates.into_iter().map(Box::new).collect())
}

/// A configuration predicate success when `all` of sub-predicates success.
pub fn all<I: IntoIterator<Item = Predicate>>(predicates: I) -> Predicate {
    Predicate::All(predicates.into_iter().map(Box::new).collect())
}

/// A configuration predicate apply `not` operator to a sub-predicate.
pub fn not(predicate: Predicate) -> Predicate {
    Predicate::Not(Box::new(predicate))
}

/// A configuration predicate with name.
pub fn name<S: Into<String>>(name: S) -> Predicate {
    Predicate::Name(name.into())
}

/// A configuration predicate with name and value.
pub fn name_value<S: Into<String>>(name: S, value: S) -> Predicate {
    Predicate::NameValue(name.into(), value.into())
}
