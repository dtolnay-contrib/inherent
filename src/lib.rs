//! [![github]](https://github.com/dtolnay/inherent)&ensp;[![crates-io]](https://crates.io/crates/inherent)&ensp;[![docs-rs]](https://docs.rs/inherent)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! <br>
//!
//! ##### An attribute macro to make trait methods callable without the trait in scope.
//!
//! # Example
//!
//! ```rust
//! mod types {
//!     use inherent::inherent;
//!
//!     trait Trait {
//!         fn f(self);
//!     }
//!
//!     pub struct Struct;
//!
//!     #[inherent(pub)]
//!     impl Trait for Struct {
//!         fn f(self) {}
//!     }
//! }
//!
//! fn main() {
//!     // types::Trait is not in scope, but method can be called.
//!     types::Struct.f();
//! }
//! ```
//!
//! Without the `inherent` macro on the trait impl, this would have failed with the
//! following error:
//!
//! ```console
//! error[E0599]: no method named `f` found for type `types::Struct` in the current scope
//!   --> src/main.rs:18:19
//!    |
//! 8  |     pub struct Struct;
//!    |     ------------------ method `f` not found for this
//! ...
//! 18 |     types::Struct.f();
//!    |                   ^
//!    |
//!    = help: items from traits can only be used if the trait is implemented and in scope
//!    = note: the following trait defines an item `f`, perhaps you need to implement it:
//!            candidate #1: `types::Trait`
//! ```
//!
//! The `inherent` macro expands to inherent methods on the `Self` type of the trait
//! impl that forward to the trait methods. In the case above, the generated code
//! would be:
//!
//! ```rust
//! # trait Trait {
//! #     fn f(self);
//! # }
//! #
//! # pub struct Struct;
//! #
//! # impl Trait for Struct {
//! #     fn f(self) {}
//! # }
//! #
//! impl Struct {
//!     pub fn f(self) {
//!         <Self as Trait>::f(self)
//!     }
//! }
//! ```
//!
//! # Visibility
//!
//! Ordinary trait methods have the same visibility as the trait or the `Self` type,
//! whichever's is smaller. Neither of these visibilities is knowable to the
//! `inherent` macro, so we simply emit all inherent methods as private by default.
//! A different visibility may be specified explicitly in the `inherent` macro
//! invocation.
//!
//! ```rust
//! # use inherent::inherent;
//! #
//! # trait Trait {}
//! #
//! # struct A;
//! #[inherent]  // private inherent methods are the default
//! # impl Trait for A {}
//!
//! # struct B;
//! #[inherent(pub)]  // all methods pub
//! # impl Trait for B {}
//!
//! # struct C;
//! #[inherent(crate)]  // all methods pub(crate)
//! # impl Trait for C {}
//!
//! # struct D;
//! #[inherent(in path::to)]  // all methods pub(in path::to)
//! # impl Trait for D {}
//! ```

#![allow(clippy::match_like_matches_macro, clippy::needless_doctest_main)]

extern crate proc_macro;

mod default_methods;
mod expand;
mod parse;
mod visibility;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::parse::TraitImpl;
use crate::visibility::Visibility;

#[proc_macro_attribute]
pub fn inherent(args: TokenStream, input: TokenStream) -> TokenStream {
    let vis = parse_macro_input!(args as Visibility);
    let input = parse_macro_input!(input as TraitImpl);
    expand::inherent(vis, input).into()
}
