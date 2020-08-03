//! Declarative testing framework

extern crate proc_macro;

use crate::block::Root;
use crate::generate::Generate;

mod block;
mod generate;

/// Allows for tests to be defined in a declarative manner, hiding repetitive code before
/// generation.
///
/// `describe`/`context` blocks define a scope such as a `mod` block and can be nested as such.
/// ```
/// # use demonstrate::demonstrate;
/// demonstrate! {
///     describe outer {
///         describe inner {}
///     }
/// }
/// ```
/// This is generated into:
/// ```
/// #[cfg(test)]
/// mod outer {
///     mod inner {}
/// }
/// ```
///
/// <hr />
///
/// `it`/`test` blocks define a unit test.
/// ```
/// # use demonstrate::demonstrate;
/// demonstrate! {
///     describe tests {
///         it asserts {
///             assert!(true)
///         }
///     }
/// }
/// ```
/// This is generated into:
/// ```
/// #[cfg(test)]
/// mod tests {
///     #[test]
///     fn asserts() {
///         assert!(true)
///     }
/// }
/// ```
///
/// <hr />
///
/// `before` and `after` blocks prevent shared starting and ending sequences of code from being
/// written for each test within a the `describe`/`context` block it is contained in and each
/// nested `describe`/`context` block.
/// ```
/// # use demonstrate::demonstrate;
/// demonstrate! {
///     describe tests {
///         before {
///             let one = 1;
///         }
///
///         it one {
///             assert_eq!(one, 1)
///         }
///
///         it zero {
///             assert_eq!(one - 1, 0)
///         }
///
///         describe nested {
///             before {
///                 let two = 2;
///             }
///
///             it two {
///                 assert_eq!(one + 1, two)
///             }
///         }
///     }
/// }
/// ```
/// This is generated into:
/// ```
/// #[cfg(test)]
/// mod tests {
///     #[test]
///     fn one() {
///         let one = 1;
///         assert_eq!(one, 1)
///     }
///
///     #[test]
///     fn zero() {
///         let one = 1;
///         assert_eq!(one - 1, 1)
///     }
///
///     mod nested {
///         #[test]
///         fn two() {
///             let one = 1;
///             let two = 2;
///             assert_eq!(one + 1, two)
///         }
///     }
/// }
/// ```
///
/// <hr />
///
/// Outer attributes, returning result types, and async tokens are all valid for `it`/`test` blocks, and can
/// be applied to `describe`/`context` blocks as well which will affect all descendant tests.
/// (Return types will only be inherited by blocks without one already defined)
/// ```
/// # use demonstrate::demonstrate;
/// demonstrate! {
///     describe returnable -> Result<(), &'static str> {
///         it is_ok { Ok(()) }
///
///         it does_not_fail {
///             assert!(!false);
///             Ok(())
///         }
///
///         #[should_panic]
///         it fails -> () {
///             assert!(false)
///         }
///     }
/// }
/// ```
/// This is generated into:
/// ```
/// #[cfg(test)]
/// mod returnable {
///     #[test]
///     fn is_ok() -> Result<(), &'static str> {
///         Ok(())
///     }
///
///     #[test]
///     fn does_not_fail() -> Result<(), &'static str> {
///         assert!(!false);
///         Ok(())
///     }
///
///     #[test]
///     #[should_panic]
///     fn fails() -> () {
///         assert!(false)
///     }
/// }
/// ```
/// **Note:** If a Describe block has a return type with an `after` block containing a success
/// result type being returned, keep in mind that a compile error will occur if a descendant test
/// has different return type than the one appearing in that `after` block.
#[proc_macro]
pub fn demonstrate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let root = syn::parse2::<Root>(input).unwrap();

    proc_macro::TokenStream::from(root.generate(None))
}
