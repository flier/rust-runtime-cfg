# runtime_cfg [![travis](https://api.travis-ci.org/flier/rust-runtime-cfg.svg)](https://travis-ci.org/flier/rust-runtime-cfg) [![crate](https://img.shields.io/crates/v/runtime_cfg.svg)](https://crates.io/crates/runtime_cfg) [![docs](https://docs.rs/runtime_cfg/badge.svg)](https://docs.rs/crate/runtime_cfg/)

Evaluation of configuration flags, at runtime-time.

## Usage

To use `runtime_cfg` in your project, add the following to your Cargo.toml:

``` toml
[dependencies]
runtime-cfg = "0.1"
```

## Example

```rust
use std::convert::TryFrom;

use quote::quote;

use runtime_cfg::*;

let cfg = quote! { #[cfg(all(unix, target_pointer_width = "32"))] };

let cfg = Cfg::try_from(cfg).unwrap();
assert_eq!(cfg, all(vec![name("unix"), name_value("target_pointer_width", "32")]).into());

let flags = vec![("unix", None), ("target_pointer_width", Some("32"))];
assert!(cfg.matches(&flags));

```
