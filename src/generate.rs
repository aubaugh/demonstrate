use crate::block::{Root, BlockType, Block};
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) trait Generate {
    fn generate(self, parent: Option<&Block>) -> TokenStream;
}

impl Generate for Root {
    fn generate(mut self, parent: Option<&Block>) -> TokenStream {
        let blocks = self
            .blocks
            .iter()
            .map(|block| block.clone().generate(parent))
            .collect::<Vec<TokenStream>>();

        quote!(
            #(#blocks)*
        )
    }
}

impl Generate for Block {
    fn generate(mut self, parent: Option<&Block>) -> TokenStream {
        if let Some(ref parent) = parent {
            self.attributes = parent
                .attributes
                .iter()
                .chain(self.attributes.iter())
                .cloned()
                .collect();

            if parent.is_async {
                self.is_async = true;
            }

            if let Some(return_type) = parent.return_type {
                self.return_type = Some(return_type);
            }
        }

        match self.content {
            BlockType::Test(mut content) => {
                let Block {
                    attributes,
                    is_async,
                    ident,
                    return_type,
                    content: _,
                } = self;

                if let Some(ref parent) = parent {
                    if let BlockType::Describe(scope) = parent.content {
                        content = scope
                            .before
                            .iter()
                            .chain(content.iter())
                            .chain(scope.after.iter())
                            .cloned()
                            .collect();
                    }
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
            BlockType::Describe(mut scope) => {
                if let Some(ref parent) = parent {
                    if let BlockType::Describe(parent_scope) = parent.content {
                        scope.before = parent_scope
                            .before
                            .iter()
                            .chain(scope.before.iter())
                            .cloned()
                            .collect();
                        scope.after.extend(parent_scope.after.clone());
                    }
                }

                let ident = self.ident;
                let scope_blocks = scope.generate(Some(&self));

                quote!(
                    #[cfg(test)]
                    mod #ident {
                        use super::*;

                        #scope_blocks
                    }
                )
            }
        }
    }
}
