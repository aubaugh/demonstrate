use proc_macro2::TokenStream;
use crate::block::{Scope, Test, Describe, Block};
use quote::quote;

pub(crate) trait Generate {
    fn generate(self, parent: Option<&Scope>) -> TokenStream;
}

impl Generate for Block {
    fn generate(self, parent: Option<&Scope>) -> TokenStream {
        match self {
            Block::Describe(block) => block.generate(parent),
            Block::Test(block) => block.generate(parent),
        }
    }
}

impl Generate for Scope {
    fn generate(mut self, parent: Option<&Scope>) -> TokenStream {
        if let Some(ref parent) = parent {
            self.before = parent.before
                .iter()
                .chain(self.before.iter())
                .cloned()
                .collect();
            self.after.extend(parent.after.clone());
        }

        let blocks = self.blocks
            .iter()
            .map(|block| block.clone().generate(Some(&self)))
            .collect::<Vec<TokenStream>>();

        quote!(
            #(#blocks)*
        )
    }
}

impl Generate for Describe {
    fn generate(self, parent: Option<&Scope>) -> TokenStream {
        let scope = self.scope.generate(parent);
        let ident = self.ident;

        quote!(
            #[cfg(test)]
            mod #ident {
                use super::*;

                #scope
            }
        )
    }
}

impl Generate for Test {
    fn generate(self, parent: Option<&Scope>) -> TokenStream {
        let ident = self.ident;
        let block = if let Some(ref parent) = parent {
            parent.before.iter()
                .chain(self.block.iter())
                .chain(parent.after.iter())
                .cloned()
                .collect()
        } else {
            self.block
        };

        quote!(
            #[test]
            fn #ident() {
                #(#block)*
            }
        )
    }
}
