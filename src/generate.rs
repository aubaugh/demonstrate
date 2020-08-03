use crate::block::*;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) trait Generate {
    fn generate(self, parent: Option<&Describe>) -> TokenStream;
}

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

impl Generate for Block {
    fn generate(self, parent: Option<&Describe>) -> TokenStream {
        match self {
            Block::Test(test) => test.generate(parent),
            Block::Describe(describe) => describe.generate(parent),
        }
    }
}

impl Generate for Describe {
    fn generate(mut self, parent: Option<&Describe>) -> TokenStream {
        self.properties.inherit(parent);

        if let Some(ref parent) = parent {
            if let Some(ref parent_before) = &parent.before {
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
                if let Some(ref mut self_after) = &mut self.after {
                    self_after.0.extend(parent_after.0.clone());
                } else {
                    self.after = Some(parent_after.clone());
                }
            }
        }

        let uses = self
            .uses
            .iter()
            .map(|path| quote!(use #path;))
            .collect::<TokenStream>();

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

impl Generate for Test {
    fn generate(mut self, parent: Option<&Describe>) -> TokenStream {
        self.properties.inherit(parent);

        let content = if let Some(ref parent) = parent {
            let mut content = if let Some(ref parent_before) = &parent.before {
                parent_before.0.clone()
            } else {
                Vec::new()
            };

            let parent_after = if let Some(ref parent_after) = &parent.after {
                parent_after.0.clone()
            } else {
                Vec::new()
            };

            content.extend(self.content.0);
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
    fn inherit(&mut self, parent: Option<&Describe>) {
        if let Some(ref parent) = parent {
            self.attributes = parent
                .properties
                .attributes
                .iter()
                .chain(self.attributes.iter())
                .cloned()
                .collect();

            if parent.properties.is_async {
                self.is_async = true;
            }

            if let Some(ref return_type) = &parent.properties.return_type {
                self.return_type = Some(return_type.clone());
            }
        }
    }
}
