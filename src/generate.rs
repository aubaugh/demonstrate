//! Defines the code translations for the various macro components

use crate::block::*;
use crate::inherit::Inherit;
use proc_macro2::TokenStream;
use quote::quote;

/// Trait and respective function for generating the corresponding code translations
pub(crate) trait Generate {
    fn generate(self, parent: Option<&Describe>) -> TokenStream;
}

/// Generates the root describe blocks within the macro, adding the `#[cfg(test)]` outer attribute
/// to each
impl Generate for Root {
    fn generate(self, _parent: Option<&Describe>) -> TokenStream {
        let Self(blocks) = self;

        blocks
            .iter()
            .map(|block| {
                let root_block = block.clone().generate(None);
                quote! {
                    #[cfg(test)]
                    #root_block
                }
            })
            .collect::<TokenStream>()
    }
}

/// Determines the respective generate function to call for each block type
impl Generate for Block {
    fn generate(self, parent: Option<&Describe>) -> TokenStream {
        match self {
            Block::Test(test) => test.generate(parent),
            Block::Describe(describe) => describe.generate(parent),
        }
    }
}

/// Generates a `mod` block with inherited block properties, inherited `before` and `after` code
/// sequences, and contained `use` paths and subblocks
impl Generate for Describe {
    fn generate(mut self, parent: Option<&Describe>) -> TokenStream {
        if let Some(ref parent) = parent {
            self.inherit(parent);
        }

        // Generate corresponding `use` statements
        let uses = self
            .uses
            .iter()
            .map(|path| quote!(use #path;))
            .collect::<TokenStream>();
        // Generate corresponding subblocks
        let blocks = self
            .blocks
            .iter()
            .map(|block| block.clone().generate(Some(&self)))
            .collect::<TokenStream>();
        let ident = self.properties.ident;
        quote! {
            mod #ident {
                #uses

                #blocks
            }
        }
    }
}

/// Generates a unit test with inherited block properties and inherited `before`/`after` code
/// sequences
impl Generate for Test {
    fn generate(mut self, parent: Option<&Describe>) -> TokenStream {
        if let Some(ref parent) = parent {
            self.inherit(parent);
        }

        let BlockProperties {
            attributes,
            is_async,
            ident,
            return_type,
        } = self.properties;
        let content = self.content.0;

        // Generate the outer attributes and optional `async` token for this test
        let (attr_tokens, async_token) = if is_async {
            (quote!(#(#attributes)*), Some(quote!(async)))
        } else {
            (
                quote! {
                    #[test]
                    #(#attributes)*
                },
                None,
            )
        };

        // Generate the test with or without a return type
        if let Some(return_type) = return_type {
            quote! {
                #attr_tokens
                #async_token fn #ident() -> #return_type {
                    #(#content)*
                }
            }
        } else {
            quote! {
                #attr_tokens
                #async_token fn #ident() {
                    #(#content)*
                }
            }
        }
    }
}
