use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, Attribute, Ident, Stmt, Token, Type};

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(before);
    custom_keyword!(after);
    custom_keyword!(context);
    custom_keyword!(describe);
    custom_keyword!(it);
    custom_keyword!(test);
}

pub(crate) enum ScopeBlock {
    Regular(Block),
    Before(Vec<Stmt>),
    After(Vec<Stmt>),
}

impl Parse for ScopeBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let content;

        if lookahead.peek(keyword::before) {
            input.parse::<keyword::before>()?;
            braced!(content in input);

            Ok(ScopeBlock::Before(content.call(syn::Block::parse_within)?))
        } else if lookahead.peek(keyword::after) {
            input.parse::<keyword::after>()?;
            braced!(content in input);

            Ok(ScopeBlock::After(content.call(syn::Block::parse_within)?))
        } else {
            Ok(ScopeBlock::Regular(input.parse::<Block>()?))
        }
    }
}

#[derive(Clone)]
pub(crate) struct Scope {
    pub(crate) before: Vec<Stmt>,
    pub(crate) after: Vec<Stmt>,
    pub(crate) blocks: Vec<Block>,
}

impl Parse for Scope {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut before = Vec::new();
        let mut after = Vec::new();
        let mut blocks = Vec::new();

        while !input.is_empty() {
            let block = input.parse::<ScopeBlock>()?;
            match block {
                ScopeBlock::Before(block) => {
                    if before.is_empty() {
                        before = block;
                    } else {
                        return Err(input.error("Only one `before` statement per scope"));
                    }
                }
                ScopeBlock::After(block) => {
                    if after.is_empty() {
                        after = block;
                    } else {
                        return Err(input.error("Only one `after` statement per scope"));
                    }
                }
                ScopeBlock::Regular(block) => blocks.push(block),
            }
        }

        Ok(Self {
            before,
            after,
            blocks,
        })
    }
}

#[derive(Clone)]
pub(crate) enum BlockType {
    Describe(Scope),
    Test(Vec<Stmt>),
}

#[derive(Clone)]
pub(crate) struct Block {
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) is_async: bool,
    pub(crate) ident: Ident,
    pub(crate) return_type: Option<Type>,
    pub(crate) content: BlockType,
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();

        let attributes = input.call(Attribute::parse_outer)?;

        let mut is_async = false;
        if lookahead.peek(Token![async]) {
            is_async = true;
            input.parse::<Token![async]>()?;
        }

        let mut is_test = false;
        if lookahead.peek(keyword::describe) {
            input.parse::<keyword::describe>()?;
        } else if lookahead.peek(keyword::context) {
            input.parse::<keyword::context>()?;
        } else if lookahead.peek(keyword::it) {
            input.parse::<keyword::it>()?;
            is_test = true;
        } else if lookahead.peek(keyword::test) {
            input.parse::<keyword::test>()?;
            is_test = true;
        }

        let ident = input.parse::<Ident>()?;

        // TODO: parse return type

        let content;
        braced!(content in input);
        let mut parsed_content;
        if is_test {
            parsed_content = BlockType::Test(content.call(syn::Block::parse_within)?);
        } else {
            parsed_content = BlockType::Describe(content.parse::<Scope>()?);
        }

        Ok(Block {
            attributes,
            is_async,
            ident,
            return_type: None,
            content: parsed_content,
        })
    }
}
