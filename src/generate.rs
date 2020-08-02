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
            .map(|block| block.clone().generate(None))
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

        let uses = self
            .uses
            .iter()
            .map(|path| quote!(use #path;))
            .collect::<TokenStream>();

        if let Some(ref parent) = parent {
            self.before = BasicBlock(
                parent
                    .before
                    .0
                    .iter()
                    .chain(self.before.0.iter())
                    .cloned()
                    .collect(),
            );
            self.after.0.extend(parent.after.0.clone());
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
        self.properties.inherit(parent);

        let content = if let Some(ref parent) = parent {
            parent
                .before
                .0
                .iter()
                .chain(self.content.0.iter())
                .chain(parent.after.0.iter())
                .cloned()
                .collect()
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
