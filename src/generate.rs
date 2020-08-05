//! Defines the code translations for the various macro components

use crate::block::*;
use crate::inherit::Inherit;
use proc_macro2::TokenStream;
use quote::quote;

/// The trait and respective function for generating the corresponding code translations
pub(crate) trait Generate {
    fn generate(&mut self, parent_props: Option<&DescribeProps>) -> TokenStream;
}

/// Generates the root `Describe` blocks within the macro, adding the `#[cfg(test)]` outer
/// attribute to each
impl Generate for Root {
    fn generate(&mut self, _parent_props: Option<&DescribeProps>) -> TokenStream {
        self.0
            .iter_mut()
            .map(|block| {
                let root_block = block.generate(None);
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
    fn generate(&mut self, parent_props: Option<&DescribeProps>) -> TokenStream {
        match self {
            Block::Test(test) => test.generate(parent_props),
            Block::Describe(describe) => describe.generate(parent_props),
        }
    }
}

/// Generates a `mod` block with inherited properties
impl Generate for Describe {
    fn generate(&mut self, parent_props: Option<&DescribeProps>) -> TokenStream {
        // Generate corresponding `use` statements
        let mut uses = self
            .properties
            .uses
            .iter()
            .map(|use_tree| quote!(use #use_tree;))
            .collect::<TokenStream>();

        // Inherit parent's `DescribeProps`
        if let Some(ref parent_props) = parent_props {
            self.inherit(parent_props);
            if !parent_props.uses.is_empty() {
                uses.extend(quote!(
                    use super::*;
                ));
            }
        }

        // Generate corresponding subblocks
        let cloned_props = self.properties.clone();
        let blocks = self
            .blocks
            .iter_mut()
            .map(|block| block.generate(Some(&cloned_props)))
            .collect::<TokenStream>();

        let ident = &self.properties.block_props.ident;
        quote! {
            mod #ident {
                #uses

                #blocks
            }
        }
    }
}

/// Generates a unit test with inherited properties
impl Generate for Test {
    fn generate(&mut self, parent_props: Option<&DescribeProps>) -> TokenStream {
        // Inherit parent's `BlockProps` and `before`/`after` code sequences
        if let Some(ref parent_props) = parent_props {
            self.inherit(parent_props);
        }

        let BlockProps {
            attributes,
            is_async,
            ident,
            return_type,
        } = &self.properties;
        let content = &self.content.0;

        // Generate the outer attributes and optional `async` token for this test
        let (attr_tokens, async_token) = if *is_async {
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
