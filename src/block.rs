use syn::parse::{Parse, ParseStream, Result};
use syn::{Attribute, Type, braced, Token, Ident, Stmt};

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(before);
    custom_keyword!(after);
    custom_keyword!(context);
    custom_keyword!(describe);
    custom_keyword!(it);
    custom_keyword!(test);
}

#[derive(Clone)]
pub(crate) enum Block {
    Describe(Describe),
    Test(Test),
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        // TODO: peek past attributes

        if lookahead.peek(Token![async]) {
            input.parse::<Token![async]>()?;
        }

        if lookahead.peek(keyword::describe) || lookahead.peek(keyword::context) {
            Ok(Block::Describe(input.parse::<Describe>()?))
        } else if lookahead.peek(keyword::it) || lookahead.peek(keyword::test) {
            Ok(Block::Test(input.parse::<Test>()?))
        } else {
            Err(input.error("Not a valid block"))
        }
    }
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
            Ok(ScopeBlock::Before(
                content.call(syn::Block::parse_within)?
            ))
        } else if lookahead.peek(keyword::after) {
            input.parse::<keyword::after>()?;
            braced!(content in input);
            Ok(ScopeBlock::After(
                content.call(syn::Block::parse_within)?
            ))
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
pub(crate) struct Describe {
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) is_async: bool,
    pub(crate) ident: Ident,
    pub(crate) return_type: Option<Type>,
    pub(crate) scope: Scope,
}

impl Parse for Describe {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut is_async = false;

        let attributes = input.call(Attribute::parse_outer)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![async]) {
            is_async = true;
            input.parse::<Token![async]>()?;
        }

        if lookahead.peek(keyword::describe) {
            input.parse::<keyword::describe>()?;
        } else if lookahead.peek(keyword::context) {
            input.parse::<keyword::context>()?;
        }

        let ident = input.parse::<Ident>()?;

        // TODO: parse return type

        let content;
        braced!(content in input);
        let scope = content.parse::<Scope>()?;

        Ok(Describe {
            attributes,
            is_async,
            ident,
            return_type: None,
            scope
        })
    }
}

#[derive(Clone)]
pub(crate) struct Test {
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) is_async: bool,
    pub(crate) ident: Ident,
    pub(crate) return_type: Option<Type>,
    pub(crate) block: Vec<Stmt>,
}

impl Parse for Test {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut is_async = false;

        let attributes = input.call(Attribute::parse_outer)?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![async]) {
            is_async = true;
            input.parse::<Token![async]>()?;
        }

        if lookahead.peek(keyword::it) {
            input.parse::<keyword::it>()?;
        } else if lookahead.peek(keyword::test) {
            input.parse::<keyword::test>()?;
        }

        let ident = input.parse::<Ident>()?;

        // TODO: parse return type

        let content;
        braced!(content in input);
        Ok(Test {
            attributes,
            is_async,
            ident,
            return_type: None,
            block: content.call(syn::Block::parse_within)?,
        })
    }
}
