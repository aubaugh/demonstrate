use crate::block::{Root, BlockProperties, Block};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) trait Generate {
    fn generate(self, parent: Option<&Describe>) -> TokenStream;
}

impl Generate for Root {
    fn generate(self, _parent: Option<&Describe>) -> TokenStream {
        let Self(blocks) = self;
        let generated_blocks = blocks
            .iter()
            .map(|block| block.clone().generate(None))
            .collect::<Vec<TokenStream>>();

        quote!(
            #(#generated_blocks)*
        )
    }
}

impl Generate for Block {
    fn generate(mut self, parent: Option<&Describe>) -> TokenStream {
        if let Some(ref parent) = parent {
            self.properties.attributes = parent
                .properties
                .attributes
                .iter()
                .chain(self.properties.attributes.iter())
                .cloned()
                .collect();

            if parent.properties.is_async {
                self.properties.is_async = true;
            }

            if let Some(return_type) = parent.properties.return_type {
                self.properties.return_type = Some(return_type);
            }
        }

        match self {
            Block::Test(mut content) => {
                let BlockProperties {
                    attributes,
                    is_async,
                    ident,
                    return_type,
                } = self.properties;

                if let Some(ref parent) = parent {
                    content = parent
                        .before
                        .iter()
                        .chain(content.iter())
                        .chain(parent.after.iter())
                        .cloned()
                        .collect();
                }

                if is_async {
                    quote!(
                        #(#attributes)*
                        async fn #ident() {
                            #(#content)*
                        }
                    )
                } else {
                    quote!(
                        #(#attributes)*
                        #[test]
                        fn #ident() {
                            #(#content)*
                        }
                    )
                }
            },
            Block::Describe(mut describe) => {
                if let Some(ref parent) = parent {
                    describe.before = parent
                        .before
                        .iter()
                        .chain(describe.before.iter())
                        .cloned()
                        .collect();
                    describe.after.extend(parent.after.clone());
                }

                let describe_blocks = describe.blocks
                    .iter()
                    .map(|block| block.clone().generate(Some(&self)))
                    .collect::<Vec<TokenStream>>();

                let ident = self.properties.ident;

                quote!(
                    #[cfg(test)]
                    mod #ident {
                        use super::*;

                        #(#describe_blocks)*
                    }
                )
            }
        }
    }
}
