use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, Attribute, Ident, Path, Stmt, Token, Type};

mod keyword {
    use syn::custom_keyword;

    custom_keyword!(before);
    custom_keyword!(after);
    custom_keyword!(context);
    custom_keyword!(describe);
    custom_keyword!(it);
    custom_keyword!(test);
}

pub(crate) struct Root(Vec<Describe>);

impl Parse for Root {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut blocks = Vec::new();

        while !input.is_empty() {
            blocks.push(input.parse::<Describe>()?);
        }

        Ok(Root(blocks))
    }
}

#[derive(Clone)]
pub(crate) enum Block {
    Describe(Describe),
    Test(Test),
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let start = input.fork();
        let _attibutes = input.call(Attribute::parse_outer)?;

        let _async_token = input.parse::<Option<Token![async]>>()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::it) || lookahead.peek(keyword::test) {
            Ok(Block::Test(start.parse::<Test>()?))
        } else if lookahead.peek(keyword::describe) || lookahead.peek(keyword::context) {
            Ok(Block::Describe(start.parse::<Describe>()?))
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Clone)]
pub(crate) struct Describe {
    pub(crate) properties: BlockProperties,
    pub(crate) uses: Vec<Path>,
    pub(crate) before: BasicBlock,
    pub(crate) after: BasicBlock,
    pub(crate) blocks: Vec<Block>,
}

impl Parse for Describe {
    fn parse(input: ParseStream) -> Result<Self> {
        let properties = input.parse::<BlockProperties>()?;

        let content;
        braced!(content in input);
        let mut uses = Vec::new();
        let mut before = BasicBlock(Vec::new());
        let mut after = BasicBlock(Vec::new());
        let mut blocks = Vec::new();

        while !content.is_empty() {
            while content.parse::<Option<Token![use]>>()?.is_some() {
                uses.push(content.call(Path::parse_mod_style)?);
                content.parse::<Token![;]>()?;
            }

            let block = content.parse::<DescribeBlock>()?;
            match block {
                DescribeBlock::Before(BasicBlock(block)) => {
                    if before.0.is_empty() {
                        before = BasicBlock(block);
                    } else {
                        return Err(
                            content.error("Only one `before` statement per describe/context scope")
                        );
                    }
                }
                DescribeBlock::After(BasicBlock(block)) => {
                    if after.0.is_empty() {
                        after = BasicBlock(block);
                    } else {
                        return Err(
                            content.error("Only one `after` statement per describe/context scope")
                        );
                    }
                }
                DescribeBlock::Regular(block) => blocks.push(block),
            }
        }

        Ok(Describe {
            properties,
            uses,
            before,
            after,
            blocks,
        })
    }
}

enum DescribeBlock {
    Regular(Block),
    Before(BasicBlock),
    After(BasicBlock),
}

impl Parse for DescribeBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.parse::<Option<keyword::before>>()?.is_some() {
            Ok(DescribeBlock::Before(input.parse::<BasicBlock>()?))
        } else if input.parse::<Option<keyword::after>>()?.is_some() {
            Ok(DescribeBlock::After(input.parse::<BasicBlock>()?))
        } else {
            Ok(DescribeBlock::Regular(input.parse::<Block>()?))
        }
    }
}

#[derive(Clone)]
pub(crate) struct Test {
    pub(crate) properties: BlockProperties,
    pub(crate) content: BasicBlock,
}

impl Parse for Test {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Test {
            properties: input.parse::<BlockProperties>()?,
            content: input.parse::<BasicBlock>()?,
        })
    }
}

#[derive(Clone)]
pub(crate) struct BasicBlock(pub(crate) Vec<Stmt>);

impl Parse for BasicBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        Ok(BasicBlock(content.call(syn::Block::parse_within)?))
    }
}

#[derive(Clone)]
pub(crate) struct BlockProperties {
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) is_async: bool,
    pub(crate) ident: Ident,
    pub(crate) return_type: Option<Type>,
}

impl Parse for BlockProperties {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;

        let is_async = input.parse::<Option<Token![async]>>()?.is_some();

        let _block_type = input.parse::<Ident>()?;

        let ident = input.parse::<Ident>()?;

        let return_type = if input.parse::<Option<Token![->]>>()?.is_some() {
            Some(input.parse::<Type>()?)
        } else {
            None
        };

        Ok(BlockProperties {
            attributes,
            is_async,
            ident,
            return_type,
        })
    }
}
