use crate::block::{Block, BlockProperties, Describe, Root, Test};
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
            .map(|block| block.clone().generate(None))
            .collect::<TokenStream>()
    }
}

impl Generate for Describe {
    fn generate(mut self, parent: Option<&Describe>) -> TokenStream {
        inherit_props(&mut self.properties, parent);

        let uses = self.uses
            .iter()
            .map(|path| quote!(use #path;))
            .collect::<TokenStream>();

        if let Some(ref parent) = parent {
            self.before = parent
                .before
                .iter()
                .chain(self.before.iter())
                .cloned()
                .collect();
            self.after.extend(parent.after.clone());
        }

        let describe_blocks = self
            .blocks
            .iter()
            .map(|block| block.clone().generate(Some(&self)))
            .collect::<TokenStream>();

        let ident = self.properties.ident;

        quote!(
            #[cfg(test)]
            mod #ident {
                #uses

                #describe_blocks
            }
        )
    }
}

impl Generate for Test {
    fn generate(mut self, parent: Option<&Describe>) -> TokenStream {
        inherit_props(&mut self.properties, parent);

        let BlockProperties {
            attributes,
            is_async,
            ident,
            return_type,
        } = self.properties;

        let content = if let Some(ref parent) = parent {
            parent
                .before
                .iter()
                .chain(self.content.iter())
                .chain(parent.after.iter())
                .cloned()
                .collect()
        } else {
            self.content
        };

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

fn inherit_props(properties: &mut BlockProperties, parent: Option<&Describe>) {
    if let Some(ref parent) = parent {
        properties.attributes = parent
            .properties
            .attributes
            .iter()
            .chain(properties.attributes.iter())
            .cloned()
            .collect();

        if parent.properties.is_async {
            properties.is_async = true;
        }

        if let Some(ref return_type) = &parent.properties.return_type {
            properties.return_type = Some(return_type.clone());
        }
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
