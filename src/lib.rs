extern crate proc_macro;
use crate::block::Scope;
use crate::generate::Generate;

mod block;
mod generate;

/* TODO:
 * Return anyhow error that is panicked or something
 * Start parsing stuff with Syn to generate tests
 * Set following restrictions:
 * - Context: Requires to have at least one test block
 * - Before & After: Only one can exist per scope
 */

#[proc_macro]
pub fn demonstrate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let root = syn::parse2::<Scope>(input).unwrap();

    //let output = quote::quote!(
    //    #[test]
    //    fn name() {
    //        assert!(true)
    //    }
    //);

    //proc_macro::TokenStream::from(output)
    proc_macro::TokenStream::from(root.generate(None))
}
