//! Defines the code translations for the various macro components

use crate::block::*;
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
                quote!(
                    #[cfg(test)]
                    #root_block
                )
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
            self.properties.inherit(parent);

            if let Some(ref parent_before) = &parent.before {
                // Prepend parent's `before` code sequence
                let before = if let Some(self_before) = self.before {
                    parent_before
                        .0
                        .iter()
                        .chain(self_before.0.iter())
                        .cloned()
                        .collect()
                } else {
                    parent_before.0.clone()
                };

                self.before = Some(BasicBlock(before));
            }

            if let Some(ref parent_after) = &parent.after {
                // Append parent's `after` code sequence
                if let Some(ref mut self_after) = &mut self.after {
                    self_after.0.extend(parent_after.0.clone());
                } else {
                    self.after = Some(parent_after.clone());
                }
            }
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
        quote!(
            mod #ident {
                #uses

                #blocks
            }
        )
    }
}

/// Generates a unit test with inherited block properties and inherited `before`/`after` code
/// sequences
impl Generate for Test {
    fn generate(mut self, parent: Option<&Describe>) -> TokenStream {
        let content = if let Some(ref parent) = parent {
            self.properties.inherit(parent);

            // Prepend parent's `before` code sequence
            let mut content = if let Some(ref parent_before) = &parent.before {
                parent_before.0.clone()
            } else {
                Vec::new()
            };
            // Add test's unique code sequence
            content.extend(self.content.0);
            // Append parent's `after` code sequence
            let parent_after = if let Some(ref parent_after) = &parent.after {
                parent_after.0.clone()
            } else {
                Vec::new()
            };
            content.extend(parent_after);
            content
        } else {
            self.content.0
        };

        let BlockProperties {
            attributes,
            is_async,
            ident,
            return_type,
        } = self.properties;

        // Generate the outer attributes and optional `async` token for this test
        let (attr_tokens, async_token) = if is_async {
            (quote!(#(#attributes)*), Some(quote!(async)))
        } else {
            (
                quote!(
                    #[test]
                    #(#attributes)*
                ),
                None,
            )
        };

        // Generate the test with or without a return type
        if let Some(return_type) = return_type {
            quote!(
                #attr_tokens
                #async_token fn #ident() -> #return_type {
                    #(#content)*
                }
            )
        } else {
            quote!(
                #attr_tokens
                #async_token fn #ident() {
                    #(#content)*
                }
            )
        }
    }
}

impl BlockProperties {
    /// Inherits the properties of the parent Describe block
    fn inherit(&mut self, parent: &Describe) {
        // Prepend parent's attributes
        self.attributes = parent
            .properties
            .attributes
            .iter()
            .chain(self.attributes.iter())
            .cloned()
            .collect();

        // If parent is async, so is self
        if parent.properties.is_async {
            self.is_async = true;
        }

        // If parent has a return type, replace self's optional return type with it
        if let Some(ref return_type) = &parent.properties.return_type {
            self.return_type = Some(return_type.clone());
        }
    }
}
