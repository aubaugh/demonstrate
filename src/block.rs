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

pub(crate) struct Root(Vec<Block>);

impl Parse for Root {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut blocks = Vec::new();

        while !input.is_empty() {
            blocks.push(input.parse::<Block>()?);
        }

        Ok(Root(blocks))
    }
}

pub(crate) enum DescribeBlock {
    Regular(Block),
    Before(Vec<Stmt>),
    After(Vec<Stmt>),
}

impl Parse for DescribeBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let content;

        if lookahead.peek(keyword::before) {
            input.parse::<keyword::before>()?;
            braced!(content in input);

            Ok(DescribeBlock::Before(content.call(syn::Block::parse_within)?))
        } else if lookahead.peek(keyword::after) {
            input.parse::<keyword::after>()?;
            braced!(content in input);

            Ok(DescribeBlock::After(content.call(syn::Block::parse_within)?))
        } else {
            Ok(DescribeBlock::Regular(input.parse::<Block>()?))
        }
    }
}

#[derive(Clone)]
pub(crate) struct Describe {
    pub(crate) properties: BlockProperties,
    pub(crate) before: Vec<Stmt>,
    pub(crate) after: Vec<Stmt>,
    pub(crate) blocks: Vec<Block>,
}

#[derive(Clone)]
pub(crate) enum Block {
    Describe(Describe),
    Test(Test),
}

#[derive(Clone)]
pub(crate) struct BlockProperties {
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) is_async: bool,
    pub(crate) ident: Ident,
    pub(crate) return_type: Option<Type>,
}

#[derive(Clone)]
pub(crate) struct Test {
    pub(crate) properties: BlockProperties,
    pub(crate) content: Vec<Stmt>,
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
        
        let properties = BlockProperties {
            attributes,
            is_async,
            ident,
            return_type: None,
        };

        let content;
        braced!(content in input);
        if is_test {
            Ok(Block::Test(Test {
                properties,
                content: content.call(syn::Block::parse_within)?
            }))
        } else {
            let mut before = Vec::new();
            let mut after = Vec::new();
            let mut blocks = Vec::new();

            while !content.is_empty() {
                let block = content.parse::<DescribeBlock>()?;
                match block {
                    DescribeBlock::Before(block) => {
                        if before.is_empty() {
                            before = block;
                        } else {
                            return Err(content.error("Only one `before` statement per describe/context scope"));
                        }
                    }
                    DescribeBlock::After(block) => {
                        if after.is_empty() {
                            after = block;
                        } else {
                            return Err(content.error("Only one `after` statement per describe/context scope"));
                        }
                    }
                    DescribeBlock::Regular(block) => blocks.push(block),
                }
            }

            Ok(Block::Describe(Describe {
                properties,
                before,
                after,
                blocks,
            }))
        }
    }
}
