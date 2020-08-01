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

enum DescribeBlock {
    Regular(Block),
    Before(Vec<Stmt>),
    After(Vec<Stmt>),
}

impl Parse for DescribeBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        if input.parse::<Option<keyword::before>>()?.is_some() {
            braced!(content in input);

            Ok(DescribeBlock::Before(
                content.call(syn::Block::parse_within)?,
            ))
        } else if input.parse::<Option<keyword::after>>()?.is_some() {
            braced!(content in input);

            Ok(DescribeBlock::After(
                content.call(syn::Block::parse_within)?,
            ))
        } else {
            Ok(DescribeBlock::Regular(input.parse::<Block>()?))
        }
    }
}

#[derive(Clone)]
pub(crate) struct Describe {
    pub(crate) before: Vec<Stmt>,
    pub(crate) after: Vec<Stmt>,
    pub(crate) blocks: Vec<Block>,
}

#[derive(Clone)]
pub(crate) struct Test(Vec<Stmt>);

#[derive(Clone)]
pub(crate) enum BlockType {
    Describe(Describe),
    Test(Vec<Stmt>),
}

#[derive(Clone)]
pub(crate) struct BlockProperties {
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) is_async: bool,
    pub(crate) ident: Ident,
    pub(crate) return_type: Option<Type>,
}

#[derive(Clone)]
pub(crate) struct Block {
    pub(crate) properties: BlockProperties,
    pub(crate) content: Vec<BlockType>,
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;

        let is_async = input.parse::<Option<Token![async]>>()?.is_some();

        let is_test = if input.parse::<Option<keyword::test>>()?.is_some()
            || input.parse::<Option<keyword::it>>()?.is_some()
        {
            true
        } else if input.parse::<Option<keyword::describe>>()?.is_some()
            || input.parse::<Option<keyword::context>>()?.is_some()
        {
            false
        } else {
            return Err(input.error("Unknown block type"));
        };

        let ident = input.parse::<Ident>()?;

        let return_type = if input.parse::<Option<Token![->]>>()?.is_some() {
            Some(input.parse::<Type>()?)
        } else {
            None
        };

        let properties = BlockProperties {
            attributes,
            is_async,
            ident,
            return_type,
        };

        let content;
        braced!(content in input);
        if is_test {
            Ok(Block::Test(Test {
                properties,
                content: content.call(syn::Block::parse_within)?,
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
                            return Err(content
                                .error("Only one `before` statement per describe/context scope"));
                        }
                    }
                    DescribeBlock::After(block) => {
                        if after.is_empty() {
                            after = block;
                        } else {
                            return Err(content
                                .error("Only one `after` statement per describe/context scope"));
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
