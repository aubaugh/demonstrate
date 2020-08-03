//! Defines the various blocks used by the `demonstrate!` macro and their corresponding `Parse`
//! implementations.

use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, Attribute, Ident, Path, Stmt, Token, Type};

/// Custom keywords used for the new blocks available in the `demonstrate!` macro
mod keyword {
    use syn::custom_keyword;

    custom_keyword!(before);

    custom_keyword!(after);

    custom_keyword!(describe);
    custom_keyword!(context);

    custom_keyword!(it);
    custom_keyword!(test);
}

/// All the describe blocks defined in the current `demonstrate!` instance
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

/// The block types that can exist in a `describe`/`context` block with `BlockProperties`
#[derive(Clone)]
pub(crate) enum Block {
    Describe(Describe),
    Test(Test),
}

impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let _attibutes = fork.call(Attribute::parse_outer)?;

        let _async_token = fork.parse::<Option<Token![async]>>()?;

        let lookahead = fork.lookahead1();
        if lookahead.peek(keyword::it) || lookahead.peek(keyword::test) {
            Ok(Block::Test(input.parse::<Test>()?))
        } else if lookahead.peek(keyword::describe) || lookahead.peek(keyword::context) {
            Ok(Block::Describe(input.parse::<Describe>()?))
        } else {
            Err(lookahead.error())
        }
    }
}

/// The `describe`/`context` block type
#[derive(Clone)]
pub(crate) struct Describe {
    /// The properties that will be passed to all descending tests
    pub(crate) properties: BlockProperties,
    /// The paths declared with `use` tokens within this block instance
    pub(crate) uses: Vec<Path>,
    /// The `before` block for this block instance
    pub(crate) before: Option<BasicBlock>,
    /// The `after` block for this block instance
    pub(crate) after: Option<BasicBlock>,
    /// The nested `describe`/`context` blocks and contained `it`/`test` blocks for this block
    /// instance
    pub(crate) blocks: Vec<Block>,
}

impl Parse for Describe {
    fn parse(input: ParseStream) -> Result<Self> {
        // Get properties
        let properties = input.parse::<BlockProperties>()?;

        // Get contents
        let content;
        braced!(content in input);
        let mut uses = Vec::new();
        let mut before = None;
        let mut after = None;
        let mut blocks = Vec::new();

        while !content.is_empty() {
            while content.parse::<Option<Token![use]>>()?.is_some() {
                uses.push(content.call(Path::parse_mod_style)?);
                content.parse::<Token![;]>()?;
            }

            let block = content.parse::<DescribeBlock>()?;
            match block {
                DescribeBlock::Before(BasicBlock(block)) => {
                    if before.is_none() {
                        before = Some(BasicBlock(block));
                    } else {
                        return Err(
                            content.error("Only one `before` statement per describe/context block")
                        );
                    }
                }
                DescribeBlock::After(BasicBlock(block)) => {
                    if after.is_none() {
                        after = Some(BasicBlock(block));
                    } else {
                        return Err(
                            content.error("Only one `after` statement per describe/context block")
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

/// The blocks permitted within a `describe`/`context` block
enum DescribeBlock {
    /// A nested Describe or Test block
    Regular(Block),
    /// A `before` block
    Before(BasicBlock),
    /// A `after` block
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

/// A `it`/`test` block
#[derive(Clone)]
pub(crate) struct Test {
    /// The properties defined for this test, or inherited from parent Describe blocks
    pub(crate) properties: BlockProperties,
    /// The unique contents of this test
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

/// Simply lines of source code that were originally within curly braces
#[derive(Clone)]
pub(crate) struct BasicBlock(pub(crate) Vec<Stmt>);

impl Parse for BasicBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        braced!(content in input);
        Ok(BasicBlock(content.call(syn::Block::parse_within)?))
    }
}

/// Properties that can apply to Describe and Test blocks
#[derive(Clone)]
pub(crate) struct BlockProperties {
    /// Outer attributes
    pub(crate) attributes: Vec<Attribute>,
    /// Whether this block or an ancestor was declared as `async`
    pub(crate) is_async: bool,
    /// The unique name for this block
    pub(crate) ident: Ident,
    /// The return type that was either defined for this block or its top-most ancestor
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
