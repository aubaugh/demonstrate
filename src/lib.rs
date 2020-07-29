extern crate proc_macro;
use crate::block::Root;
use crate::generate::Generate;

mod block;
mod generate;

/* TODO:
 * Return anyhow error that is panicked or something
 * Set following restriction:
 * - Context: Requires to have at least one test block
 */

#[proc_macro]
pub fn demonstrate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    let root = syn::parse2::<Root>(input).unwrap();

    proc_macro::TokenStream::from(root.generate(None))
}
